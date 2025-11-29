//! Rights mechanic implementation.
//!
//! This module provides the main `RightsMechanic` type, which composes
//! rights system, transfer, and recognition policies into a complete rights system.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, ParallelSafe};

use super::policies::{RecognitionPolicy, RightsSystemPolicy, TransferPolicy};
use super::strategies::{AbsoluteRights, FreeTransfer, SelfRecognition};
use super::types::*;

/// Generic rights mechanic.
///
/// This type composes three orthogonal policy dimensions:
/// - **RightsSystemPolicy** (R): How claims are structured
/// - **TransferPolicy** (T): How claims can be transferred
/// - **RecognitionPolicy** (G): How claims are validated
///
/// # Type Parameters
///
/// - `R`: Rights system policy (default: `AbsoluteRights`)
/// - `T`: Transfer policy (default: `FreeTransfer`)
/// - `G`: Recognition policy (default: `SelfRecognition`)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
/// use issun_core::mechanics::Mechanic;
///
/// // Modern property rights (absolute, freely transferable, self-recognized)
/// type PropertyRights = RightsMechanic<
///     AbsoluteRights,
///     FreeTransfer,
///     SelfRecognition,
/// >;
/// ```
pub struct RightsMechanic<
    R: RightsSystemPolicy = AbsoluteRights,
    T: TransferPolicy = FreeTransfer,
    G: RecognitionPolicy = SelfRecognition,
> {
    _marker: PhantomData<(R, T, G)>,
}

impl<R, T, G> Mechanic for RightsMechanic<R, T, G>
where
    R: RightsSystemPolicy,
    T: TransferPolicy,
    G: RecognitionPolicy,
{
    type Config = RightsConfig;
    type State = RightsState;
    type Input = RightsInput;
    type Event = RightsEvent;
    type Execution = ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // Remove expired claims first
        if input.elapsed_time > 0 {
            let expired = state.remove_expired_claims(input.elapsed_time);
            for asset_id in expired {
                emitter.emit(RightsEvent::ClaimExpired {
                    asset_id,
                    expiration_time: input.elapsed_time,
                });
            }
        }

        // Process the action
        match input.action {
            RightsAction::AssertClaim {
                asset_id,
                strength,
                expiration,
            } => {
                // Validate claim using rights system policy
                match R::validate_claim(strength, config) {
                    Ok(validated_strength) => {
                        // Create and add claim
                        let claim = if let Some(exp) = expiration {
                            Claim::with_expiration(asset_id, validated_strength, exp)
                        } else {
                            Claim::new(asset_id, validated_strength)
                        };

                        state.claims.insert(asset_id, claim);

                        emitter.emit(RightsEvent::ClaimAsserted {
                            asset_id,
                            strength: validated_strength,
                            expiration,
                        });
                    }
                    Err(reason) => {
                        emitter.emit(RightsEvent::ActionRejected {
                            action: input.action,
                            reason,
                        });
                    }
                }
            }

            RightsAction::TransferClaim {
                asset_id,
                recipient,
                amount,
            } => {
                // Check if transfer is allowed
                match T::can_transfer(state, asset_id, amount, config) {
                    Ok(()) => {
                        // Execute transfer
                        T::execute_transfer(state, asset_id, amount);

                        emitter.emit(RightsEvent::ClaimTransferred {
                            asset_id,
                            recipient,
                            amount,
                        });
                    }
                    Err(reason) => {
                        emitter.emit(RightsEvent::ActionRejected {
                            action: input.action,
                            reason,
                        });
                    }
                }
            }

            RightsAction::ChallengeClaim {
                asset_id,
                incumbent,
            } => {
                // Emit challenge event
                // Actual resolution would be handled by game logic
                emitter.emit(RightsEvent::ClaimChallenged {
                    asset_id,
                    incumbent,
                });
            }

            RightsAction::GrantRecognition {
                claimant: _,
                asset_id,
            } => {
                // Note: Recognition is granted TO another entity
                // This would be handled at the system level in Bevy
                // Here we just emit the event
                emitter.emit(RightsEvent::RecognitionGranted {
                    claimant: 0, // Placeholder - actual entity handled by game logic
                    asset_id,
                });
            }

            RightsAction::RevokeRecognition {
                claimant: _,
                asset_id,
            } => {
                emitter.emit(RightsEvent::RecognitionRevoked {
                    claimant: 0, // Placeholder
                    asset_id,
                });
            }
        }

        // Update legitimacy based on recognition
        let old_legitimacy = state.legitimacy;
        G::update_legitimacy(state, state.recognized_by.len(), config);

        // Apply decay if time elapsed
        if input.elapsed_time > 0 {
            G::apply_decay(state, input.elapsed_time, config);
        }

        // Emit legitimacy change if significant
        if (old_legitimacy - state.legitimacy).abs() > 0.01 {
            emitter.emit(RightsEvent::LegitimacyChanged {
                old_value: old_legitimacy,
                new_value: state.legitimacy,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::rights::strategies::*;

    // Test event collector
    struct TestEmitter {
        events: Vec<RightsEvent>,
    }

    impl TestEmitter {
        fn new() -> Self {
            Self { events: Vec::new() }
        }
    }

    impl EventEmitter<RightsEvent> for TestEmitter {
        fn emit(&mut self, event: RightsEvent) {
            self.events.push(event);
        }
    }

    type BasicRights = RightsMechanic<AbsoluteRights, FreeTransfer, SelfRecognition>;

    #[test]
    fn test_assert_claim_success() {
        let config = RightsConfig::default();
        let mut state = RightsState::new();
        let mut emitter = TestEmitter::new();

        let input = RightsInput {
            action: RightsAction::AssertClaim {
                asset_id: 42,
                strength: 1.0,
                expiration: None,
            },
            elapsed_time: 0,
        };

        BasicRights::step(&config, &mut state, input, &mut emitter);

        assert!(state.has_claim(42));
        assert_eq!(state.claim_strength(42), 1.0);
        assert!(emitter
            .events
            .iter()
            .any(|e| matches!(&e, RightsEvent::ClaimAsserted { .. })));
    }

    #[test]
    fn test_assert_claim_with_expiration() {
        let config = RightsConfig::default();
        let mut state = RightsState::new();
        let mut emitter = TestEmitter::new();

        let input = RightsInput {
            action: RightsAction::AssertClaim {
                asset_id: 42,
                strength: 1.0,
                expiration: Some(100),
            },
            elapsed_time: 0,
        };

        BasicRights::step(&config, &mut state, input, &mut emitter);

        assert!(state.has_claim(42));
        assert_eq!(state.claims.get(&42).unwrap().expiration_time, Some(100));
    }

    #[test]
    fn test_assert_partial_claim_rejected_by_absolute_rights() {
        let config = RightsConfig::default();
        let mut state = RightsState::new();
        let mut emitter = TestEmitter::new();

        let input = RightsInput {
            action: RightsAction::AssertClaim {
                asset_id: 42,
                strength: 0.5,
                expiration: None,
            },
            elapsed_time: 0,
        };

        BasicRights::step(&config, &mut state, input, &mut emitter);

        assert!(!state.has_claim(42));
        assert!(emitter
            .events
            .iter()
            .any(|e| matches!(&e, RightsEvent::ActionRejected { .. })));
    }

    #[test]
    fn test_transfer_claim_success() {
        let config = RightsConfig::default();
        let mut state = RightsState::new();
        state.claims.insert(42, Claim::new(42, 1.0));
        let mut emitter = TestEmitter::new();

        let input = RightsInput {
            action: RightsAction::TransferClaim {
                asset_id: 42,
                recipient: 99,
                amount: 1.0,
            },
            elapsed_time: 0,
        };

        BasicRights::step(&config, &mut state, input, &mut emitter);

        // Entire claim transferred (removed from state)
        assert!(!state.has_claim(42));
        assert!(emitter
            .events
            .iter()
            .any(|e| matches!(&e, RightsEvent::ClaimTransferred { .. })));
    }

    #[test]
    fn test_transfer_insufficient_claim() {
        let config = RightsConfig::default();
        let mut state = RightsState::new();
        state.claims.insert(42, Claim::new(42, 0.5));
        let mut emitter = TestEmitter::new();

        let input = RightsInput {
            action: RightsAction::TransferClaim {
                asset_id: 42,
                recipient: 99,
                amount: 1.0, // More than owned
            },
            elapsed_time: 0,
        };

        BasicRights::step(&config, &mut state, input, &mut emitter);

        // Transfer rejected, claim unchanged
        assert_eq!(state.claim_strength(42), 0.5);
        assert!(emitter
            .events
            .iter()
            .any(|e| matches!(&e, RightsEvent::ActionRejected { .. })));
    }

    #[test]
    fn test_claim_expiration() {
        let config = RightsConfig::default();
        let mut state = RightsState::new();
        state
            .claims
            .insert(42, Claim::with_expiration(42, 1.0, 100));
        let mut emitter = TestEmitter::new();

        let input = RightsInput {
            action: RightsAction::AssertClaim {
                asset_id: 99,
                strength: 1.0,
                expiration: None,
            },
            elapsed_time: 150, // Past expiration
        };

        BasicRights::step(&config, &mut state, input, &mut emitter);

        // Expired claim removed
        assert!(!state.has_claim(42));
        assert!(emitter
            .events
            .iter()
            .any(|e| matches!(&e, RightsEvent::ClaimExpired { .. })));
    }

    #[test]
    fn test_partial_rights_system() {
        type PartialRightsSystem = RightsMechanic<PartialRights, FreeTransfer, SelfRecognition>;

        let config = RightsConfig {
            allow_partial_claims: true,
            ..Default::default()
        };
        let mut state = RightsState::new();
        let mut emitter = TestEmitter::new();

        let input = RightsInput {
            action: RightsAction::AssertClaim {
                asset_id: 42,
                strength: 0.5,
                expiration: None,
            },
            elapsed_time: 0,
        };

        PartialRightsSystem::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.claim_strength(42), 0.5);
    }

    #[test]
    fn test_legitimacy_change() {
        type RecognizedRights = RightsMechanic<PartialRights, FreeTransfer, AuthorityRecognition>;

        let config = RightsConfig {
            allow_partial_claims: true,
            require_recognition: true,
            ..Default::default()
        };
        let mut state = RightsState::new();
        let mut emitter = TestEmitter::new();

        let input = RightsInput {
            action: RightsAction::AssertClaim {
                asset_id: 42,
                strength: 1.0,
                expiration: None,
            },
            elapsed_time: 0,
        };

        RecognizedRights::step(&config, &mut state, input, &mut emitter);

        // Without recognition, legitimacy should be 0
        assert_eq!(state.legitimacy, 0.0);
    }
}
