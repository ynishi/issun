//! The core SecuritizationMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic};

use super::policies::{CollateralPolicy, IssuancePolicy};
use super::strategies::{FullBackingIssuance, SimpleCollateral};
use super::types::{
    SecuritizationAction, SecuritizationConfig, SecuritizationEvent, SecuritizationInput,
    SecuritizationState,
};

/// The core securitization mechanic that composes collateral and issuance policies.
///
/// # Type Parameters
///
/// - `C`: Collateral policy (manages asset locking and redemption)
/// - `I`: Issuance policy (manages security issuance limits and backing)
pub struct SecuritizationMechanic<
    C: CollateralPolicy = SimpleCollateral,
    I: IssuancePolicy = FullBackingIssuance,
> {
    _marker: PhantomData<(C, I)>,
}

impl<C, I> Mechanic for SecuritizationMechanic<C, I>
where
    C: CollateralPolicy,
    I: IssuancePolicy,
{
    type Config = SecuritizationConfig;
    type State = SecuritizationState;
    type Input = SecuritizationInput;
    type Event = SecuritizationEvent;
    type Execution = crate::mechanics::ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        match input.action {
            SecuritizationAction::Lock => {
                Self::handle_lock(config, state, input, emitter);
            }
            SecuritizationAction::Issue => {
                Self::handle_issue(config, state, input, emitter);
            }
            SecuritizationAction::Redeem => {
                Self::handle_redeem(config, state, input, emitter);
            }
        }
    }
}

impl<C, I> SecuritizationMechanic<C, I>
where
    C: CollateralPolicy,
    I: IssuancePolicy,
{
    fn handle_lock(
        config: &SecuritizationConfig,
        state: &mut SecuritizationState,
        input: SecuritizationInput,
        emitter: &mut impl EventEmitter<SecuritizationEvent>,
    ) {
        // Verify asset can be locked
        if let Err(reason) =
            C::can_lock_asset(input.asset_value, state.collateral_value, state.is_locked)
        {
            emitter.emit(SecuritizationEvent::OperationRejected {
                action: SecuritizationAction::Lock,
                reason,
            });
            return;
        }

        // Calculate collateral value
        let collateral_value =
            C::calculate_collateral_value(input.asset_value, input.risk_factor, config);

        // Lock the asset
        state.collateral_value += collateral_value;
        state.update_backing_ratio();

        emitter.emit(SecuritizationEvent::AssetLocked {
            value: collateral_value,
            total_collateral: state.collateral_value,
        });

        emitter.emit(SecuritizationEvent::BackingRatioUpdated {
            old_ratio: if state.issued_securities > 0.0 {
                (state.collateral_value - collateral_value) / state.issued_securities
            } else {
                f32::INFINITY
            },
            new_ratio: state.backing_ratio,
        });
    }

    fn handle_issue(
        config: &SecuritizationConfig,
        state: &mut SecuritizationState,
        input: SecuritizationInput,
        emitter: &mut impl EventEmitter<SecuritizationEvent>,
    ) {
        // Verify securities can be issued
        if let Err(reason) = I::can_issue_securities(
            input.securities_amount,
            state.collateral_value,
            state.issued_securities,
            state.backing_ratio,
            state.is_locked,
            config,
        ) {
            emitter.emit(SecuritizationEvent::OperationRejected {
                action: SecuritizationAction::Issue,
                reason,
            });
            return;
        }

        // Calculate fee
        let fee = I::calculate_issuance_fee(input.securities_amount, config);

        // Issue securities
        let old_ratio = state.backing_ratio;
        state.issued_securities += input.securities_amount;
        state.update_backing_ratio();

        emitter.emit(SecuritizationEvent::SecuritiesIssued {
            amount: input.securities_amount,
            fee,
            total_issued: state.issued_securities,
        });

        emitter.emit(SecuritizationEvent::BackingRatioUpdated {
            old_ratio,
            new_ratio: state.backing_ratio,
        });
    }

    fn handle_redeem(
        config: &SecuritizationConfig,
        state: &mut SecuritizationState,
        input: SecuritizationInput,
        emitter: &mut impl EventEmitter<SecuritizationEvent>,
    ) {
        // Check if enough securities exist
        if input.securities_amount > state.issued_securities {
            emitter.emit(SecuritizationEvent::OperationRejected {
                action: SecuritizationAction::Redeem,
                reason: super::types::RejectionReason::InsufficientSecurities,
            });
            return;
        }

        // Calculate redemption value
        let collateral_returned = C::calculate_redemption_value(
            input.securities_amount,
            state.collateral_value,
            state.issued_securities,
            config,
        );

        // Calculate fee
        let fee = I::calculate_redemption_fee(input.securities_amount, config);

        // Redeem securities
        let old_ratio = state.backing_ratio;
        state.issued_securities -= input.securities_amount;
        state.collateral_value -= collateral_returned;
        state.update_backing_ratio();

        emitter.emit(SecuritizationEvent::SecuritiesRedeemed {
            amount: input.securities_amount,
            collateral_returned,
            fee,
        });

        emitter.emit(SecuritizationEvent::BackingRatioUpdated {
            old_ratio,
            new_ratio: state.backing_ratio,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct VecEmitter(Vec<SecuritizationEvent>);
    impl EventEmitter<SecuritizationEvent> for VecEmitter {
        fn emit(&mut self, event: SecuritizationEvent) {
            self.0.push(event);
        }
    }

    type SimpleSecuritization = SecuritizationMechanic;

    #[test]
    fn test_lock_asset() {
        let config = SecuritizationConfig::default();
        let mut state = SecuritizationState::default();
        let input = SecuritizationInput {
            action: SecuritizationAction::Lock,
            asset_value: 1000.0,
            securities_amount: 0.0,
            risk_factor: 0.0,
        };

        let mut emitter = VecEmitter(vec![]);
        SimpleSecuritization::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.collateral_value, 1000.0);
        assert!(emitter
            .0
            .iter()
            .any(|e| matches!(e, SecuritizationEvent::AssetLocked { value: 1000.0, .. })));
    }

    #[test]
    fn test_issue_securities() {
        let config = SecuritizationConfig::default();
        let mut state = SecuritizationState::new(1000.0);
        state.update_backing_ratio();

        let input = SecuritizationInput {
            action: SecuritizationAction::Issue,
            asset_value: 0.0,
            securities_amount: 500.0,
            risk_factor: 0.0,
        };

        let mut emitter = VecEmitter(vec![]);
        SimpleSecuritization::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.issued_securities, 500.0);
        assert!(emitter.0.iter().any(|e| matches!(
            e,
            SecuritizationEvent::SecuritiesIssued { amount: 500.0, .. }
        )));
    }

    #[test]
    fn test_redeem_securities() {
        let config = SecuritizationConfig::default();
        let mut state = SecuritizationState::new(1000.0);
        state.issued_securities = 500.0;
        state.update_backing_ratio();

        let input = SecuritizationInput {
            action: SecuritizationAction::Redeem,
            asset_value: 0.0,
            securities_amount: 250.0,
            risk_factor: 0.0,
        };

        let mut emitter = VecEmitter(vec![]);
        SimpleSecuritization::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.issued_securities, 250.0);
        assert_eq!(state.collateral_value, 500.0); // 1000 - (250/500)*1000 = 500
    }
}
