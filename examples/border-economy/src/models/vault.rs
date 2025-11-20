use super::context::{DiplomaticAction, PolicyCard, PolicyEffects};
use super::ids::{BudgetChannel, Currency, VaultId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VaultStatus {
    Active,
    Peril { weeks_remaining: u8 },
    Captured { lockout_weeks: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlotType {
    Research,
    Security,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlotEffect {
    RnDBuff {
        progress_bonus: f32,
        telemetry_bonus: f32,
    },
    OpsRelief {
        hostility_drop: f32,
        ops_cost_multiplier: f32,
    },
    SpecialCard {
        card_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSlot {
    pub slot_id: String,
    pub name: String,
    pub slot_type: SlotType,
    pub base_threshold: Currency,
    pub max_threshold: Currency,
    pub decay_rate: f32,
    pub current_investment: Currency,
    pub effect: SlotEffect,
    pub active: bool,
}

impl VaultSlot {
    pub fn apply_investment(&mut self, amount: Currency) -> (Currency, Currency) {
        let before = self.current_investment;
        let mut after_value = before.amount().saturating_add(amount.amount());
        if after_value > self.max_threshold.amount() {
            after_value = self.max_threshold.amount();
        }
        self.current_investment = Currency::new(after_value);
        self.active = self.current_investment.amount() >= self.base_threshold.amount();
        (before, self.current_investment)
    }

    pub fn tick_week(&mut self) -> Currency {
        if self.current_investment.amount() <= 0 {
            self.active = false;
            return Currency::ZERO;
        }
        let decay_amount = ((self.current_investment.amount() as f32) * self.decay_rate)
            .round()
            .max(0.0) as i64;
        if decay_amount <= 0 {
            return Currency::ZERO;
        }
        let deduction = Currency::new(decay_amount);
        self.current_investment -= deduction;
        if self.current_investment.amount() < self.base_threshold.amount() {
            self.active = false;
        }
        deduction
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: VaultId,
    pub codename: String,
    pub discovery_week: u32,
    pub volatility: f32,
    pub defense_window_weeks: u8,
    pub security_requirement: Currency,
    pub slots: Vec<VaultSlot>,
    pub status: VaultStatus,
    pub pending_special_cards: Vec<PolicyCard>,
    pub peril_counter: u8,
    pub last_assault: Option<String>,
}

impl Vault {
    pub fn demo() -> Self {
        Self::spark_ridge()
    }

    pub fn templates() -> Vec<Self> {
        vec![
            Self::spark_ridge(),
            Self::nova_cache(),
            Self::mythic_omega(),
        ]
    }

    fn spark_ridge() -> Self {
        Self {
            id: VaultId::new("spark-ridge-vault"),
            codename: "Spark Ridge Vault".into(),
            discovery_week: 1,
            volatility: 0.6,
            defense_window_weeks: 3,
            security_requirement: Currency::new(900),
            status: VaultStatus::Active,
            slots: vec![
                VaultSlot {
                    slot_id: "proto_accel".into(),
                    name: "Prototype Accelerator".into(),
                    slot_type: SlotType::Research,
                    base_threshold: Currency::new(600),
                    max_threshold: Currency::new(1200),
                    decay_rate: 0.08,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::RnDBuff {
                        progress_bonus: 0.12,
                        telemetry_bonus: 0.05,
                    },
                    active: false,
                },
                VaultSlot {
                    slot_id: "orbital_bastion".into(),
                    name: "Orbital Bastion".into(),
                    slot_type: SlotType::Security,
                    base_threshold: Currency::new(400),
                    max_threshold: Currency::new(1000),
                    decay_rate: 0.06,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::OpsRelief {
                        hostility_drop: 0.15,
                        ops_cost_multiplier: 0.92,
                    },
                    active: false,
                },
                VaultSlot {
                    slot_id: "emerald_key".into(),
                    name: "Emerald Key Access".into(),
                    slot_type: SlotType::Special,
                    base_threshold: Currency::new(700),
                    max_threshold: Currency::new(1500),
                    decay_rate: 0.1,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::SpecialCard {
                        card_id: "emerald_consignment".into(),
                    },
                    active: false,
                },
            ],
            pending_special_cards: vec![PolicyCard {
                id: "emerald_consignment".into(),
                name: "Emerald Consignment".into(),
                description: "Emerald Consort専用の高額契約を獲得。外交アクションが増える".into(),
                effects: PolicyEffects {
                    dividend_multiplier: 1.05,
                    investment_bonus: 1.2,
                    ops_cost_multiplier: 0.95,
                    diplomacy_bonus: 1.3,
                },
                available_actions: vec![DiplomaticAction::JointResearch],
            }],
            peril_counter: 0,
            last_assault: None,
        }
    }

    fn nova_cache() -> Self {
        Self {
            id: VaultId::new("nova-cache"),
            codename: "Nova Cache".into(),
            discovery_week: 3,
            volatility: 0.45,
            defense_window_weeks: 4,
            security_requirement: Currency::new(1200),
            status: VaultStatus::Active,
            slots: vec![
                VaultSlot {
                    slot_id: "telemetry_array".into(),
                    name: "Telemetry Array".into(),
                    slot_type: SlotType::Research,
                    base_threshold: Currency::new(800),
                    max_threshold: Currency::new(1500),
                    decay_rate: 0.09,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::RnDBuff {
                        progress_bonus: 0.15,
                        telemetry_bonus: 0.08,
                    },
                    active: false,
                },
                VaultSlot {
                    slot_id: "orbital_shield".into(),
                    name: "Orbital Shield".into(),
                    slot_type: SlotType::Security,
                    base_threshold: Currency::new(600),
                    max_threshold: Currency::new(1400),
                    decay_rate: 0.07,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::OpsRelief {
                        hostility_drop: 0.18,
                        ops_cost_multiplier: 0.88,
                    },
                    active: false,
                },
                VaultSlot {
                    slot_id: "diplomatic_brief".into(),
                    name: "Diplomatic Brief".into(),
                    slot_type: SlotType::Special,
                    base_threshold: Currency::new(750),
                    max_threshold: Currency::new(1400),
                    decay_rate: 0.08,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::SpecialCard {
                        card_id: "nova_embassy".into(),
                    },
                    active: false,
                },
            ],
            pending_special_cards: vec![PolicyCard {
                id: "nova_embassy".into(),
                name: "Nova Embassy".into(),
                description: "外交特権カード: 敵対勢力との関係改善ボーナス".into(),
                effects: PolicyEffects {
                    dividend_multiplier: 1.0,
                    investment_bonus: 1.1,
                    ops_cost_multiplier: 0.9,
                    diplomacy_bonus: 1.5,
                },
                available_actions: vec![DiplomaticAction::LimitedCooperation],
            }],
            peril_counter: 0,
            last_assault: None,
        }
    }

    fn mythic_omega() -> Self {
        Self {
            id: VaultId::new("omega-vault"),
            codename: "Mythic Vault Omega".into(),
            discovery_week: 5,
            volatility: 0.85,
            defense_window_weeks: 2,
            security_requirement: Currency::new(1500),
            status: VaultStatus::Active,
            slots: vec![
                VaultSlot {
                    slot_id: "prototype_forge".into(),
                    name: "Prototype Forge".into(),
                    slot_type: SlotType::Research,
                    base_threshold: Currency::new(900),
                    max_threshold: Currency::new(1800),
                    decay_rate: 0.12,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::RnDBuff {
                        progress_bonus: 0.2,
                        telemetry_bonus: 0.12,
                    },
                    active: false,
                },
                VaultSlot {
                    slot_id: "titan_guard".into(),
                    name: "Titan Guard".into(),
                    slot_type: SlotType::Security,
                    base_threshold: Currency::new(800),
                    max_threshold: Currency::new(1800),
                    decay_rate: 0.1,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::OpsRelief {
                        hostility_drop: 0.25,
                        ops_cost_multiplier: 0.85,
                    },
                    active: false,
                },
                VaultSlot {
                    slot_id: "legendary_map".into(),
                    name: "Legendary Map".into(),
                    slot_type: SlotType::Special,
                    base_threshold: Currency::new(1000),
                    max_threshold: Currency::new(2000),
                    decay_rate: 0.12,
                    current_investment: Currency::ZERO,
                    effect: SlotEffect::SpecialCard {
                        card_id: "omega_accord".into(),
                    },
                    active: false,
                },
            ],
            pending_special_cards: vec![PolicyCard {
                id: "omega_accord".into(),
                name: "Omega Accord".into(),
                description: "危険な契約。Opsコスト減とInnovation大幅増".into(),
                effects: PolicyEffects {
                    dividend_multiplier: 1.2,
                    investment_bonus: 1.4,
                    ops_cost_multiplier: 0.8,
                    diplomacy_bonus: 0.9,
                },
                available_actions: vec![DiplomaticAction::WarningStrike],
            }],
            peril_counter: 0,
            last_assault: None,
        }
    }

    pub fn tick_week(&mut self) -> VaultReport {
        let mut total_investment = Currency::ZERO;
        let mut decay_total = Currency::ZERO;
        let mut active_slots = 0usize;
        for slot in &mut self.slots {
            total_investment += slot.current_investment;
            decay_total += slot.tick_week();
            if slot.active {
                active_slots += 1;
            }
        }

        let mut report = VaultReport {
            vault_id: self.id.clone(),
            codename: self.codename.clone(),
            status: self.status.clone(),
            active_slots,
            total_investment,
            decay_applied: decay_total,
            warnings: self.generate_warnings(),
            assault_log: self.last_assault.clone(),
            outcome: None,
        };

        self.update_status();
        report.status = self.status.clone();
        report.warnings = self.generate_warnings();
        report.assault_log = self.last_assault.take();
        report.outcome = self.roll_outcome(active_slots);
        report
    }

    fn roll_outcome(&self, active_slots: usize) -> Option<VaultOutcome> {
        if !matches!(self.status, VaultStatus::Active) {
            return None;
        }

        let slot_factor = (active_slots as f32 / self.slots.len().max(1) as f32).clamp(0.0, 1.0);
        let risk = self.volatility.clamp(0.1, 1.2);
        let rand = ((self.discovery_week * 17 + active_slots as u32 * 13) % 100) as f32 / 100.0;
        let roll = slot_factor * 0.6 + rand * 0.4 - risk * 0.2;

        if roll > 0.85 {
            Some(VaultOutcome::Jackpot {
                credits: Currency::new(1200 + (800.0 * slot_factor) as i64),
                reputation: 12,
            })
        } else if roll > 0.65 {
            Some(VaultOutcome::Success {
                credits: Currency::new(700 + (500.0 * slot_factor) as i64),
            })
        } else if roll > 0.4 {
            Some(VaultOutcome::Mediocre {
                credits: Currency::new(250 + (250.0 * slot_factor) as i64),
                antiquities: (slot_factor * 3.0).round() as i32,
            })
        } else if roll > 0.15 {
            Some(VaultOutcome::Disaster {
                debt: Currency::new(300 + (200.0 * risk) as i64),
                casualties: (30.0 * risk) as u32,
            })
        } else {
            Some(VaultOutcome::Catastrophe {
                reputation_penalty: -15,
            })
        }
    }

    fn update_status(&mut self) {
        match &mut self.status {
            VaultStatus::Captured { lockout_weeks } => {
                if *lockout_weeks > 0 {
                    *lockout_weeks -= 1;
                }
                if *lockout_weeks == 0 {
                    self.status = VaultStatus::Active;
                    self.peril_counter = 0;
                }
                return;
            }
            _ => {}
        }

        let security_investment: i64 = self
            .slots
            .iter()
            .filter(|slot| matches!(slot.slot_type, SlotType::Security))
            .map(|slot| slot.current_investment.amount())
            .sum();
        if (matches!(self.status, VaultStatus::Active)
            || matches!(self.status, VaultStatus::Peril { .. }))
            && security_investment < self.security_requirement.amount()
        {
            let rand = ((self.discovery_week + self.peril_counter as u32 * 7) % 100) as f32 / 100.0;
            self.peril_counter = self.peril_counter.saturating_add(1);
            let risk: f32 = (1.0 - (security_investment as f32 / self.security_requirement.amount() as f32))
                .clamp(0.0, 1.0);
            let assault_roll = self.volatility * risk * rand;
            if assault_roll > 0.4 {
                self.last_assault = Some("敵勢力がVaultを強襲".into());
                if self.peril_counter >= self.defense_window_weeks.saturating_sub(1) {
                    self.capture_vault();
                } else {
                    self.status = VaultStatus::Peril {
                        weeks_remaining: self
                            .defense_window_weeks
                            .saturating_sub(self.peril_counter)
                            .max(1),
                    };
                }
                return;
            }

            if self.peril_counter >= self.defense_window_weeks {
                self.capture_vault();
            } else {
                let weeks_remaining = self
                    .defense_window_weeks
                    .saturating_sub(self.peril_counter)
                    .max(1);
                self.status = VaultStatus::Peril { weeks_remaining };
            }
        } else {
            self.peril_counter = 0;
            self.status = VaultStatus::Active;
        }
    }

    fn capture_vault(&mut self) {
        self.status = VaultStatus::Captured { lockout_weeks: 3 };
        self.last_assault = Some("Vaultが敵勢力に占拠された".into());
        for slot in &mut self.slots {
            slot.current_investment = Currency::ZERO;
            slot.active = false;
        }
    }

    fn generate_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if matches!(self.status, VaultStatus::Peril { .. }) {
            warnings.push("Vault is under enemy pressure".into());
        }
        let coverage = self
            .slots
            .iter()
            .filter(|slot| matches!(slot.slot_type, SlotType::Security))
            .map(|slot| slot.current_investment.amount())
            .sum::<i64>();
        if coverage < self.security_requirement.amount() / 2 {
            warnings.push("Security coverage不足".into());
        }
        warnings
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultReport {
    pub vault_id: VaultId,
    pub codename: String,
    pub status: VaultStatus,
    pub active_slots: usize,
    pub total_investment: Currency,
    pub decay_applied: Currency,
    pub warnings: Vec<String>,
    pub assault_log: Option<String>,
    pub outcome: Option<VaultOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInvestmentResult {
    pub vault_id: VaultId,
    pub slot_id: String,
    pub amount: Currency,
    pub before: Currency,
    pub after: Currency,
    pub activated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultInvestmentError {
    VaultNotFound,
    SlotNotFound,
    InvalidAmount,
    ChannelMismatch {
        expected: BudgetChannel,
        provided: BudgetChannel,
    },
    InsufficientFunds {
        required: Currency,
        channel: BudgetChannel,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultOutcome {
    Jackpot { credits: Currency, reputation: i32 },
    Success { credits: Currency },
    Mediocre { credits: Currency, antiquities: i32 },
    Disaster { debt: Currency, casualties: u32 },
    Catastrophe { reputation_penalty: i32 },
}
