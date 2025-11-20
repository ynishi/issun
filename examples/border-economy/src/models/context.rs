use super::ids::{
    BudgetChannel, Currency, DemandProfile, FactionId, ReputationStanding, TerritoryId, VaultId,
    WeaponPrototypeId,
};
use super::vault::{
    SlotEffect, SlotType, Vault, VaultInvestmentError, VaultInvestmentResult, VaultOutcome,
    VaultReport, VaultStatus,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub const SETTLEMENT_PERIOD_DAYS: u32 = 7;
pub const DAILY_ACTION_POINTS: u32 = 3;
pub const DIVIDEND_BASE: i64 = 200;
pub const DIVIDEND_RATE: f32 = 0.04;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEffects {
    pub dividend_multiplier: f32,
    pub investment_bonus: f32,
    pub ops_cost_multiplier: f32,
    pub diplomacy_bonus: f32,
}

impl PolicyEffects {
    pub fn default() -> Self {
        Self {
            dividend_multiplier: 1.0,
            investment_bonus: 1.0,
            ops_cost_multiplier: 1.0,
            diplomacy_bonus: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: PolicyEffects,
    pub available_actions: Vec<DiplomaticAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiplomaticAction {
    WarningStrike,
    LimitedCooperation,
    JointResearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionProfile {
    pub id: FactionId,
    pub codename: String,
    pub readiness: u8,
    pub deployed: bool,
    pub doctrine: Doctrine,
}

impl FactionProfile {
    pub fn new(id: &str, codename: &str, readiness: u8, doctrine: Doctrine) -> Self {
        Self {
            id: FactionId::new(id),
            codename: codename.to_string(),
            readiness,
            deployed: false,
            doctrine,
        }
    }

    pub fn mark_deployment(&mut self) {
        self.deployed = true;
        self.readiness = self.readiness.saturating_sub(5).max(30);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doctrine {
    pub aggression: u8,
    pub logistics: u8,
}

impl Doctrine {
    pub fn frontier_raiders() -> Self {
        Self {
            aggression: 80,
            logistics: 40,
        }
    }

    pub fn civic_guard() -> Self {
        Self {
            aggression: 40,
            logistics: 75,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryIntel {
    pub id: TerritoryId,
    pub control: f32,
    pub unrest: f32,
    pub demand: DemandProfile,
    pub logistics_rating: f32,
    pub enemy_share: f32,
    pub conflict_intensity: f32,
    pub battlefront: bool,
    pub enemy_faction: EnemyFactionId,
    pub development_level: f32,
    pub market_tier: u8,
    pub pending_investment: f32,
}

impl TerritoryIntel {
    pub fn new(
        id: &str,
        control: f32,
        unrest: f32,
        demand: DemandProfile,
        enemy_faction: EnemyFactionId,
    ) -> Self {
        Self {
            id: TerritoryId::new(id),
            control,
            unrest,
            demand,
            logistics_rating: 0.5,
            enemy_share: 1.0 - control,
            conflict_intensity: 0.5,
            battlefront: false,
            enemy_faction,
            development_level: 0.2,
            market_tier: 1,
            pending_investment: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponPrototypeState {
    pub id: WeaponPrototypeId,
    pub codename: String,
    pub progress: f32,
    pub quality: f32,
    pub telemetry: f32,
}

impl WeaponPrototypeState {
    pub fn new(id: &str, codename: &str) -> Self {
        Self {
            id: WeaponPrototypeId::new(id),
            codename: codename.to_string(),
            progress: 0.35,
            quality: 0.5,
            telemetry: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLedger {
    pub cash: Currency,
    pub research_pool: Currency,
    pub ops_pool: Currency,
    pub reserve: Currency,
    pub innovation_fund: Currency,
    pub security_fund: Currency,
}

impl BudgetLedger {
    pub fn new(starting_cash: Currency) -> Self {
        Self {
            cash: starting_cash,
            research_pool: Currency::new(600),
            ops_pool: Currency::new(600),
            reserve: Currency::new(400),
            innovation_fund: Currency::new(0),
            security_fund: Currency::new(0),
        }
    }

    pub fn channel_mut(&mut self, channel: BudgetChannel) -> &mut Currency {
        match channel {
            BudgetChannel::Research => &mut self.research_pool,
            BudgetChannel::Operations => &mut self.ops_pool,
            BudgetChannel::Reserve => &mut self.reserve,
            BudgetChannel::Innovation => &mut self.innovation_fund,
            BudgetChannel::Security => &mut self.security_fund,
        }
    }

    pub fn channel_ref(&self, channel: BudgetChannel) -> &Currency {
        match channel {
            BudgetChannel::Research => &self.research_pool,
            BudgetChannel::Operations => &self.ops_pool,
            BudgetChannel::Reserve => &self.reserve,
            BudgetChannel::Innovation => &self.innovation_fund,
            BudgetChannel::Security => &self.security_fund,
        }
    }

    pub fn shift(&mut self, from: BudgetChannel, to: BudgetChannel, delta: Currency) -> bool {
        if from == to {
            return false;
        }
        let from_balance = *self.channel_mut(from);
        if from_balance.amount() < delta.amount() {
            return false;
        }
        *self.channel_mut(from) -= delta;
        *self.channel_mut(to) += delta;
        true
    }

    pub fn can_spend(&self, channel: BudgetChannel, amount: Currency) -> bool {
        self.channel_ref(channel).amount() >= amount.amount()
    }

    pub fn try_spend(&mut self, channel: BudgetChannel, amount: Currency) -> bool {
        let balance = self.channel_mut(channel);
        if balance.amount() < amount.amount() {
            return false;
        }
        *balance -= amount;
        true
    }

    pub fn transfer_from_cash(&mut self, amount: Currency) -> bool {
        if self.cash.amount() < amount.amount() {
            return false;
        }
        self.cash -= amount;
        true
    }

    pub fn direct_invest(&mut self, channel: BudgetChannel, amount: Currency) {
        let target = self.channel_mut(channel);
        *target += amount;
    }

    pub fn innovation_multiplier(&self) -> f32 {
        ((self.innovation_fund.amount() as f32) / 2000.0).clamp(0.0, 0.35)
    }

    pub fn investment_income_bonus(&self) -> i64 {
        ((self.innovation_fund.amount() as f32) * 0.05) as i64
    }

    pub fn security_upkeep_offset(&self) -> i64 {
        ((self.security_fund.amount() as f32) * 0.08) as i64
    }

    pub fn apply_investment_decay(&mut self) -> Option<String> {
        let mut notes = Vec::new();
        let innovation_loss = ((self.innovation_fund.amount() as f32) * 0.08) as i64;
        if innovation_loss > 0 {
            let deduction = Currency::new(innovation_loss);
            if deduction.amount() > 0 {
                self.innovation_fund -= deduction;
                notes.push(format!("Innovation維持費 {}", deduction));
            }
        }

        let security_loss = ((self.security_fund.amount() as f32) * 0.05) as i64;
        if security_loss > 0 {
            let deduction = Currency::new(security_loss);
            if deduction.amount() > 0 {
                self.security_fund -= deduction;
                notes.push(format!("Security維持費 {}", deduction));
            }
        }

        if notes.is_empty() {
            None
        } else {
            Some(notes.join(" / "))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAllocation {
    pub channel: BudgetChannel,
    pub amount: Currency,
}

impl BudgetAllocation {
    pub fn new(channel: BudgetChannel, amount: Currency) -> Self {
        Self { channel, amount }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SettlementResult {
    pub net: Currency,
    pub reserve_bonus: Currency,
    pub innovation_allocation: Currency,
    pub security_allocation: Currency,
    pub ops_spent: Currency,
    pub rnd_spent: Currency,
    pub dev_spent: Currency,
}

#[derive(Debug, Clone, Copy)]
pub struct DividendEventResult {
    pub demanded: Currency,
    pub paid_from_reserve: Currency,
    pub paid_from_cash: Currency,
    pub shortfall: Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameContext {
    pub day: u32,
    // ledger moved to issun::plugin::BudgetLedger resource
    pub factions: Vec<FactionProfile>,
    pub territories: Vec<TerritoryIntel>,
    pub prototypes: Vec<WeaponPrototypeState>,
    pub enemy_factions: Vec<EnemyFaction>,
    pub reputation: ReputationStanding,
    pub recent_log: Vec<String>,
    pub actions_remaining: u32,
    pub last_enemy_action: Option<String>,
    pub pending_logs: Vec<String>,
    pub enemy_operations: Vec<EnemyOperation>,
    pub policies: Vec<PolicyCard>,
    pub active_policy_index: usize,
    pub weekly_ops_spent: Currency,
    pub weekly_rnd_spent: Currency,
    pub weekly_dev_spent: Currency,
    pub vaults: Vec<Vault>,
    pub last_vault_reports: Vec<VaultReport>,
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            day: 1,
            // ledger initialization moved to issun::plugin::BuiltInEconomyPlugin
            factions: vec![
                FactionProfile::new(
                    "rust_runners",
                    "Rust Runners",
                    70,
                    Doctrine::frontier_raiders(),
                ),
                FactionProfile::new(
                    "civic_lancers",
                    "Civic Lancers",
                    55,
                    Doctrine::civic_guard(),
                ),
            ],
            territories: vec![
                TerritoryIntel::new(
                    "spark-ridge",
                    0.35,
                    0.6,
                    DemandProfile::frontier(),
                    EnemyFactionId::new("rust_band"),
                ),
                TerritoryIntel::new(
                    "nova-harbor",
                    0.55,
                    0.3,
                    DemandProfile::metroplex(),
                    EnemyFactionId::new("arbiter_order"),
                ),
                TerritoryIntel::new(
                    "glowmarsh",
                    0.45,
                    0.5,
                    DemandProfile::frontier(),
                    EnemyFactionId::new("emerald_consort"),
                ),
                TerritoryIntel::new(
                    "azure-hold",
                    0.6,
                    0.25,
                    DemandProfile::metroplex(),
                    EnemyFactionId::new("azure_kartel"),
                ),
            ],
            prototypes: vec![
                WeaponPrototypeState::new("xr-31", "XR-31 Rail Darter"),
                WeaponPrototypeState::new("sunburst", "SUNBURST crowd pacifier"),
            ],
            enemy_factions: vec![
                EnemyFaction::new(
                    "rust_band",
                    "Dust Claws",
                    0.9,
                    "砂漠の重装騎兵。激しい突撃で支配域を奪う",
                ),
                EnemyFaction::new(
                    "arbiter_order",
                    "Order of Iron",
                    0.6,
                    "規律正しい治安維持組織。じわじわと支配域を拡大",
                ),
                EnemyFaction::new(
                    "emerald_consort",
                    "Emerald Consort",
                    0.5,
                    "巨大企業が支援する準合法コングロマリット。外交工作が巧み",
                ),
                EnemyFaction::new(
                    "azure_kartel",
                    "Azure Kartel",
                    0.8,
                    "港湾都市を支配する密輸カルテル。資金力で攻勢をかける",
                ),
            ],
            reputation: ReputationStanding::default(),
            recent_log: vec!["起業家評議会: 初期投資を受領".into()],
            actions_remaining: DAILY_ACTION_POINTS,
            last_enemy_action: None,
            pending_logs: Vec::new(),
            enemy_operations: Vec::new(),
            policies: vec![
                PolicyCard {
                    id: "appease_investors".into(),
                    name: "投資家優遇策".into(),
                    description: "配当要求が増すが、市場信頼度が向上し投資効果が高まる".into(),
                    effects: PolicyEffects {
                        dividend_multiplier: 1.2,
                        investment_bonus: 1.3,
                        ops_cost_multiplier: 1.0,
                        diplomacy_bonus: 0.9,
                    },
                    available_actions: vec![DiplomaticAction::JointResearch],
                },
                PolicyCard {
                    id: "security_surge".into(),
                    name: "治安強化キャンペーン".into(),
                    description: "Opsコストが増えるが、敵攻勢を抑制できる".into(),
                    effects: PolicyEffects {
                        dividend_multiplier: 0.9,
                        investment_bonus: 1.0,
                        ops_cost_multiplier: 0.85,
                        diplomacy_bonus: 1.0,
                    },
                    available_actions: vec![DiplomaticAction::WarningStrike],
                },
                PolicyCard {
                    id: "expansion_drive".into(),
                    name: "開拓優先令".into(),
                    description: "Opsコストが増すが開拓投資効率が上昇".into(),
                    effects: PolicyEffects {
                        dividend_multiplier: 1.0,
                        investment_bonus: 1.5,
                        ops_cost_multiplier: 1.1,
                        diplomacy_bonus: 1.2,
                    },
                    available_actions: vec![DiplomaticAction::LimitedCooperation],
                },
            ],
            active_policy_index: 0,
            weekly_ops_spent: Currency::ZERO,
            weekly_rnd_spent: Currency::ZERO,
            weekly_dev_spent: Currency::ZERO,
            vaults: Vault::templates(),
            last_vault_reports: Vec::new(),
        }
    }

    pub fn record(&mut self, line: impl Into<String>) {
        self.recent_log
            .insert(0, format!("Day {}: {}", self.day, line.into()));
        self.recent_log.truncate(6);
    }

    pub fn pick_ready_faction(&self) -> Option<FactionProfile> {
        self.factions
            .iter()
            .filter(|f| !f.deployed)
            .max_by_key(|f| f.readiness)
            .cloned()
    }

    pub fn mark_faction_deployed(&mut self, id: &FactionId) {
        if let Some(faction) = self.factions.iter_mut().find(|f| &f.id == id) {
            faction.mark_deployment();
        }
    }

    pub fn territory_snapshot(&self, id: &TerritoryId) -> Option<TerritoryIntel> {
        self.territories.iter().find(|t| &t.id == id).cloned()
    }

    pub fn enemy_faction_by_id(&self, id: &EnemyFactionId) -> Option<&EnemyFaction> {
        self.enemy_factions.iter().find(|f| &f.id == id)
    }

    pub fn active_policy(&self) -> &PolicyCard {
        self.policies
            .get(self.active_policy_index)
            .unwrap_or_else(|| &self.policies[0])
    }

    pub fn cycle_policy(&mut self) -> &PolicyCard {
        self.active_policy_index = (self.active_policy_index + 1) % self.policies.len();
        let idx = self.active_policy_index;
        let name = self.policies[idx].name.clone();
        self.record(format!("政策「{}」を採択", name));
        &self.policies[idx]
    }

    pub fn adjust_control(&mut self, id: &TerritoryId, delta: f32) {
        if let Some(territory) = self.territories.iter_mut().find(|t| &t.id == id) {
            territory.control = (territory.control + delta).clamp(0.0, 1.0);
            territory.unrest = (territory.unrest - delta).clamp(0.0, 1.0);
            territory.enemy_share = (1.0 - territory.control).clamp(0.0, 1.0);
            territory.conflict_intensity = (territory.conflict_intensity * 0.85).clamp(0.1, 1.0);
            if territory.enemy_share < 0.2 {
                territory.battlefront = false;
            }
        }
    }

    // apply_revenue removed - now done directly via issun::plugin::BudgetLedger
    // (see tactical.rs for example)

    pub fn record_ops_spend(&mut self, amount: Currency) {
        self.weekly_ops_spent += amount;
    }

    pub fn record_rnd_spend(&mut self, amount: Currency) {
        self.weekly_rnd_spent += amount;
    }

    pub fn record_dev_spend(&mut self, amount: Currency) {
        self.weekly_dev_spent += amount;
    }

    pub fn increment_day(&mut self) {
        self.flush_pending_logs();
        self.day += 1;
        self.actions_remaining = DAILY_ACTION_POINTS;
        for faction in &mut self.factions {
            faction.deployed = false;
            faction.readiness = (faction.readiness + 4).min(100);
        }
        if let Some(report) = self.update_enemy_operations() {
            self.last_enemy_action = Some(report.clone());
            self.record(report);
        } else {
            self.last_enemy_action = None;
        }

        self.schedule_enemy_operation();
        self.tick_development();
    }

    pub fn consume_action(&mut self, context: &str) -> bool {
        if self.actions_remaining == 0 {
            self.record("行動ポイントが残っていません。自動的に翌日に進みます。");
            self.increment_day();
            return true;
        }

        self.actions_remaining -= 1;
        if self.actions_remaining == 0 {
            self.record(format!(
                "{}を実施し、日次行動をすべて消費しました。",
                context
            ));
            self.increment_day();
            true
        } else {
            false
        }
    }

    pub fn force_end_of_day(&mut self, reason: impl Into<String>) {
        self.record(reason);
        self.flush_pending_logs();
        self.increment_day();
    }

    pub fn action_status(&self) -> (u32, u32) {
        (self.actions_remaining, DAILY_ACTION_POINTS)
    }

    // NOTE: process_dividend_event moved to economy.rs plugin
    // It now operates directly on issun::plugin::BudgetLedger from ResourceContext

    fn flush_pending_logs(&mut self) {
        let logs = self.pending_logs.drain(..).collect::<Vec<_>>();
        for log in logs {
            self.record(log);
        }
    }

    pub fn invest_in_territory(&mut self, territory_id: &TerritoryId, amount: Currency) -> bool {
        if self.ledger.innovation_fund.amount() < amount.amount() {
            return false;
        }
        self.ledger.innovation_fund -= amount;
        let bonus = self.active_policy().effects.investment_bonus;
        self.record_dev_spend(amount);
        if let Some(territory) = self.territories.iter_mut().find(|t| &t.id == territory_id) {
            territory.pending_investment += (amount.amount() as f32 / 1000.0) * bonus;
            self.pending_logs.push(format!(
                "{} にインフラ投資 ({} )",
                territory.id.as_str(),
                amount
            ));
            true
        } else {
            false
        }
    }

    fn tick_development(&mut self) {
        for territory in &mut self.territories {
            if territory.pending_investment > 0.0 {
                let increment = (territory.pending_investment * 0.1).clamp(0.0, 0.15);
                territory.pending_investment = (territory.pending_investment - increment).max(0.0);
                territory.development_level =
                    (territory.development_level + increment).clamp(0.0, 1.0);
                if territory.development_level > (territory.market_tier as f32 * 0.3 + 0.3) {
                    territory.market_tier = territory.market_tier.saturating_add(1).min(5);
                    let message = format!(
                        "{} が開拓レベル{}に到達",
                        territory.id.as_str(),
                        territory.market_tier
                    );
                    self.pending_logs.push(message);
                }
            }
        }
    }

    pub fn vaults(&self) -> &[Vault] {
        &self.vaults
    }

    pub fn invest_in_vault_slot(
        &mut self,
        vault_id: &VaultId,
        slot_id: &str,
        amount: Currency,
        channel: BudgetChannel,
    ) -> Result<VaultInvestmentResult, VaultInvestmentError> {
        if amount.amount() <= 0 {
            return Err(VaultInvestmentError::InvalidAmount);
        }

        let vault = self
            .vaults
            .iter_mut()
            .find(|vault| &vault.id == vault_id)
            .ok_or(VaultInvestmentError::VaultNotFound)?;

        let slot = vault
            .slots
            .iter_mut()
            .find(|slot| slot.slot_id == slot_id)
            .ok_or(VaultInvestmentError::SlotNotFound)?;

        let expected_channel = match slot.slot_type {
            SlotType::Security => BudgetChannel::Security,
            SlotType::Research | SlotType::Special => BudgetChannel::Innovation,
        };

        if expected_channel != channel {
            return Err(VaultInvestmentError::ChannelMismatch {
                expected: expected_channel,
                provided: channel,
            });
        }

        if !self.ledger.try_spend(channel, amount) {
            return Err(VaultInvestmentError::InsufficientFunds {
                required: amount,
                channel,
            });
        }

        let (before, after) = slot.apply_investment(amount);
        let activated = slot.active;

        self.pending_logs.push(format!(
            "Vault {} の {} へ {} 投資",
            vault.codename, slot.name, amount
        ));

        Ok(VaultInvestmentResult {
            vault_id: vault.id.clone(),
            slot_id: slot.slot_id.clone(),
            amount,
            before,
            after,
            activated,
        })
    }

    pub fn tick_vaults(&mut self) -> Vec<VaultReport> {
        if self.vaults.is_empty() {
            return Vec::new();
        }

        let reports = self
            .vaults
            .iter_mut()
            .map(|vault| vault.tick_week())
            .collect::<Vec<_>>();
        self.last_vault_reports = reports.clone();
        self.apply_vault_effects();
        reports
    }

    pub fn apply_vault_effects(&mut self) {
        let reports = self.last_vault_reports.clone();
        for report in &reports {
            if let Some(index) = self.vaults.iter().position(|v| v.id == report.vault_id) {
                let (vault_codename, status, volatility) = {
                    let vault = &self.vaults[index];
                    (
                        vault.codename.clone(),
                        vault.status.clone(),
                        vault.volatility,
                    )
                };

                if let Some(log) = &report.assault_log {
                    self.pending_logs
                        .push(format!("Vault {}: {}", vault_codename, log));
                }

                match status {
                    VaultStatus::Captured { lockout_weeks } => {
                        let penalty = Currency::new((200.0 * volatility).round() as i64);
                        self.ledger.ops_pool -= penalty;
                        self.reputation.adjust(-4.0);
                        self.pending_logs.push(format!(
                            "Vault {} を失いOps {}喪失 (再稼働まで{}週)",
                            vault_codename, penalty, lockout_weeks
                        ));
                        continue;
                    }
                    VaultStatus::Peril { weeks_remaining } => {
                        self.pending_logs.push(format!(
                            "Vault {} が危機状態: Security再投資まで残り{}週",
                            vault_codename, weeks_remaining
                        ));
                    }
                    VaultStatus::Active => {}
                }

                if let Some(outcome) = &report.outcome {
                    self.resolve_vault_outcome(outcome);
                }

                let slots = self.vaults[index].slots.clone();
                for slot in &slots {
                    if !slot.active {
                        continue;
                    }
                    match slot.effect {
                        SlotEffect::RnDBuff {
                            progress_bonus,
                            telemetry_bonus,
                        } => {
                            if let Some(proto) = self.prototypes.first_mut() {
                                proto.progress = (proto.progress + progress_bonus).min(1.0);
                                proto.telemetry = (proto.telemetry + telemetry_bonus).min(1.0);
                                self.pending_logs.push(format!(
                                    "Vault {} がR&Dにブースト (+{:.0}%/+{:.0}%)",
                                    vault_codename,
                                    progress_bonus * 100.0,
                                    telemetry_bonus * 100.0
                                ));
                            }
                        }
                        SlotEffect::OpsRelief {
                            hostility_drop,
                            ops_cost_multiplier,
                        } => {
                            for faction in &mut self.enemy_factions {
                                faction.hostility =
                                    (faction.hostility - hostility_drop).clamp(0.1, 2.5);
                            }
                            // Treat multiplier as rebate percentage applied to recent Ops spend.
                            if self.weekly_ops_spent.amount() > 0 {
                                let rebate = (self.weekly_ops_spent.amount() as f32
                                    * (1.0 - ops_cost_multiplier))
                                    .max(0.0) as i64;
                                if rebate > 0 {
                                    let credit = Currency::new(rebate);
                                    self.ledger.ops_pool += credit;
                                    self.pending_logs.push(format!(
                                        "Vault {} の防衛ラインでOpsコスト{}返還",
                                        vault_codename, credit
                                    ));
                                }
                            }
                        }
                        SlotEffect::SpecialCard { ref card_id } => {
                            if self.policies.iter().any(|policy| policy.id == *card_id) {
                                continue;
                            }

                            if let Some(vault) = self.vaults.get_mut(index) {
                                if let Some(idx) = vault
                                    .pending_special_cards
                                    .iter()
                                    .position(|card| &card.id == card_id)
                                {
                                    let card = vault.pending_special_cards.remove(idx);
                                    self.policies.push(card.clone());
                                    self.pending_logs.push(format!(
                                        "Vault {} で特別カード{}を獲得",
                                        vault_codename, card.name
                                    ));
                                    continue;
                                }
                            }

                            self.pending_logs.push(format!(
                                "Vault {} が特別契約 {} を示唆",
                                vault_codename, card_id
                            ));
                        }
                    }
                }
            }
        }
    }

    fn resolve_vault_outcome(&mut self, outcome: &VaultOutcome) {
        match outcome {
            VaultOutcome::Jackpot {
                credits,
                reputation,
            } => {
                self.ledger.cash += *credits;
                self.reputation.adjust(*reputation as f32);
                self.pending_logs
                    .push(format!("Vault投資でJACKPOT! +{}", credits));
            }
            VaultOutcome::Success { credits } => {
                self.ledger.cash += *credits;
                self.pending_logs.push(format!("Vault収益 +{}", credits));
            }
            VaultOutcome::Mediocre {
                credits,
                antiquities,
            } => {
                self.ledger.cash += *credits;
                self.pending_logs.push(format!(
                    "Vault調査: 収益 {} / 遺物{}点",
                    credits, antiquities
                ));
            }
            VaultOutcome::Disaster { debt, casualties } => {
                self.ledger.cash -= *debt;
                self.reputation.adjust(-5.0);
                self.pending_logs.push(format!(
                    "Vault事故で損失 {} / Casualties {}",
                    debt, casualties
                ));
            }
            VaultOutcome::Catastrophe { reputation_penalty } => {
                self.reputation.adjust(*reputation_penalty as f32);
                self.pending_logs
                    .push("Vault調査が大惨事に… Reputation急落".into());
            }
        }
    }

    fn schedule_enemy_operation(&mut self) {
        if self.territories.is_empty() {
            return;
        }

        let front_index = self
            .territories
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let a_score = a.enemy_share * a.conflict_intensity;
                let b_score = b.enemy_share * b.conflict_intensity;
                a_score.partial_cmp(&b_score).unwrap_or(Ordering::Equal)
            })
            .map(|(idx, _)| idx);

        let Some(front_index) = front_index else {
            return;
        };

        for (idx, territory) in self.territories.iter_mut().enumerate() {
            territory.battlefront = idx == front_index;
        }

        let front_id = self.territories[front_index].id.clone();
        if self
            .enemy_operations
            .iter()
            .any(|op| op.territory == front_id)
        {
            return;
        }

        let front = &self.territories[front_index];
        let (aggression, hostility) = self
            .enemy_faction_by_id(&front.enemy_faction)
            .map(|f| {
                (
                    (f.aggression - f.relations).clamp(0.1, 1.2),
                    f.hostility.clamp(0.1, 1.0),
                )
            })
            .unwrap_or((0.7, 0.5));
        let intensity = ((0.05 + front.enemy_share * 0.1 + front.conflict_intensity * 0.07)
            * (0.7 + aggression + hostility))
            .clamp(0.03, 0.4);
        let min_eta = ((3.5 - aggression * 2.0).max(1.0)).round() as u32;
        let eta = (2 + (front.conflict_intensity * 3.0) as u32).max(min_eta);
        self.enemy_operations.push(EnemyOperation {
            territory: front.id.clone(),
            eta,
            intensity,
            faction: front.enemy_faction.clone(),
        });
    }

    fn update_enemy_operations(&mut self) -> Option<String> {
        let mut completed = Vec::new();
        for op in self.enemy_operations.iter_mut() {
            if op.eta > 0 {
                op.eta -= 1;
            }
            if op.eta == 0 {
                completed.push(op.clone());
            }
        }

        self.enemy_operations.retain(|op| op.eta > 0);

        let mut last_report = None;
        for op in completed {
            if let Some(report) = self.resolve_enemy_operation(&op) {
                last_report = Some(report);
            }
        }

        self.roll_enemy_events();

        last_report
    }

    fn resolve_enemy_operation(&mut self, op: &EnemyOperation) -> Option<String> {
        let faction = self.enemy_faction_by_id(&op.faction);
        let aggression = faction
            .map(|f| (f.aggression - f.relations).clamp(0.1, 1.2))
            .unwrap_or(0.7);
        let faction_name = faction
            .map(|f| f.codename.as_str())
            .unwrap_or("未知の勢力")
            .to_string();

        let territory = self.territories.iter_mut().find(|t| t.id == op.territory)?;

        let mitigation = (self.ledger.security_fund.amount() as f32 / 2500.0).clamp(0.0, 0.5);
        let effective = (op.intensity * (1.0 - mitigation)).clamp(0.01, 0.3);

        territory.control = (territory.control - effective).clamp(0.0, 1.0);
        territory.enemy_share = (territory.enemy_share + effective).clamp(0.0, 1.0);
        territory.unrest = (territory.unrest + effective * 0.7).clamp(0.0, 1.0);
        territory.conflict_intensity = (territory.conflict_intensity + 0.12).clamp(0.0, 1.0);
        territory.battlefront = true;

        Some(format!(
            "{} が{}で攻勢 ({:.0}%激化): 支配率 -{:.1}% (Security防衛 {:.0}%)",
            faction_name,
            territory.id.as_str(),
            aggression * 100.0,
            effective * 100.0,
            mitigation * 100.0
        ))
    }

    fn roll_enemy_events(&mut self) {
        for faction in &mut self.enemy_factions {
            let hostility = faction.hostility + (1.0 - faction.relations.max(-0.9));
            if hostility > 1.6 {
                faction.last_event = Some("攻勢声明: 前線に追加攻勢を準備".into());
                faction.hostility = (faction.hostility + 0.1).min(2.0);
                self.pending_logs
                    .push(format!("{} が攻勢声明を発表", faction.codename));
            } else if faction.relations > 0.6 {
                faction.last_event = Some("共同開発提案: 技術交換の余地あり".into());
                faction.hostility = (faction.hostility * 0.9).max(0.1);
                self.pending_logs
                    .push(format!("{} が共同開発を提案", faction.codename));
            } else {
                faction.last_event = None;
            }
        }
    }

    pub fn settlement_due(&self) -> bool {
        self.day % SETTLEMENT_PERIOD_DAYS == 0
    }

    pub fn forecast_income(&self) -> Currency {
        let total = self
            .territories
            .iter()
            .map(|territory| {
                let control = territory.control;
                let stability = 1.0 - territory.unrest;
                let logistics = territory.demand.logistics_weight;
                ((control * 650.0) + (stability * 400.0) + (logistics * 250.0)) as i64
            })
            .sum::<i64>();
        Currency::new(total + self.ledger.investment_income_bonus())
    }

    pub fn forecast_upkeep(&self) -> Currency {
        let faction_cost = (self.factions.len() as i64) * 120;
        let prototype_cost = (self.prototypes.len() as i64) * 80;
        let security = self
            .territories
            .iter()
            .map(|t| ((t.unrest + 0.1) * 90.0) as i64)
            .sum::<i64>();
        let base = faction_cost + prototype_cost + security;
        let reduction = self.ledger.security_upkeep_offset();
        Currency::new((base - reduction).max(0))
    }

    pub fn apply_settlement(&mut self, income: Currency, upkeep: Currency) -> SettlementResult {
        self.ledger.cash += income;
        self.ledger.cash -= upkeep;
        let net_amount = income.amount() - upkeep.amount();
        let mut reserve_bonus = Currency::new(0);
        let mut innovation_allocation = Currency::new(0);
        let mut security_allocation = Currency::new(0);

        if net_amount > 0 {
            reserve_bonus = Currency::new((net_amount as f32 * 0.25) as i64);
            if reserve_bonus.amount() > 0 {
                self.ledger.reserve += reserve_bonus;
            }

            let invest_total = Currency::new((net_amount as f32 * 0.3) as i64);
            if invest_total.amount() > 0 {
                innovation_allocation = Currency::new((invest_total.amount() as f32 * 0.6) as i64);
                security_allocation = Currency::new(invest_total.amount() - innovation_allocation.amount());
                if innovation_allocation.amount() > 0 {
                    self.ledger.innovation_fund += innovation_allocation;
                }
                if security_allocation.amount() > 0 {
                    self.ledger.security_fund += security_allocation;
                }
            }
        }

        let net = Currency::new(net_amount);
        self.record(format!(
            "決算処理 収益{} - コスト{} → 純益{}",
            income, upkeep, net
        ));
        if innovation_allocation.amount() > 0 || security_allocation.amount() > 0 {
            self.record(format!(
                "自動投資: Innovation {} / Security {}",
                innovation_allocation, security_allocation
            ));
        }
        if let Some(note) = self.ledger.apply_investment_decay() {
            self.record(note);
        }
        let ops_spent = self.weekly_ops_spent;
        let rnd_spent = self.weekly_rnd_spent;
        let dev_spent = self.weekly_dev_spent;
        self.weekly_ops_spent = Currency::ZERO;
        self.weekly_rnd_spent = Currency::ZERO;
        self.weekly_dev_spent = Currency::ZERO;

        SettlementResult {
            net,
            reserve_bonus,
            innovation_allocation,
            security_allocation,
            ops_spent,
            rnd_spent,
            dev_spent,
        }
    }

    pub fn queue_research(&mut self, id: &WeaponPrototypeId, delta: f32) {
        if let Some(proto) = self.prototypes.iter_mut().find(|p| &p.id == id) {
            let multiplier = 1.0 + self.ledger.innovation_multiplier();
            proto.progress = (proto.progress + delta * multiplier).min(1.0);
            proto.quality = (proto.quality + delta * 0.5 * multiplier).min(1.0);
        }
    }

    pub fn push_telemetry(&mut self, id: &WeaponPrototypeId, payload: f32) {
        if let Some(proto) = self.prototypes.iter_mut().find(|p| &p.id == id) {
            proto.telemetry = (proto.telemetry + payload).clamp(0.0, 1.0);
        }
    }
}

impl Default for GameContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyOperation {
    pub territory: TerritoryId,
    pub eta: u32,
    pub intensity: f32,
    pub faction: EnemyFactionId,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnemyFactionId(String);

impl EnemyFactionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyFaction {
    pub id: EnemyFactionId,
    pub codename: String,
    pub aggression: f32,
    pub description: String,
    pub relations: f32,
    pub hostility: f32,
    pub last_event: Option<String>,
}

impl EnemyFaction {
    pub fn new(
        id: impl Into<String>,
        codename: impl Into<String>,
        aggression: f32,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: EnemyFactionId::new(id),
            codename: codename.into(),
            aggression,
            description: description.into(),
            relations: 0.0,
            hostility: 0.5,
            last_event: None,
        }
    }
}

impl GameContext {
    pub fn apply_campaign_response(&mut self, faction_id: &EnemyFactionId, relations_delta: f32) {
        if self.enemy_factions.iter().any(|f| &f.id == faction_id) {
            let diplomacy_bonus = self.active_policy().effects.diplomacy_bonus;
            let mut impacted = Vec::new();
            if let Some(faction) = self.enemy_factions.iter_mut().find(|f| &f.id == faction_id) {
                faction.relations =
                    (faction.relations + relations_delta * diplomacy_bonus).clamp(-1.0, 1.0);
                impacted.push((faction.codename.clone(), faction.relations));
                faction.hostility = (faction.hostility - relations_delta * 0.2).clamp(0.1, 2.0);
            }

            for rival in self
                .enemy_factions
                .iter_mut()
                .filter(|f| &f.id != faction_id)
            {
                rival.hostility = (rival.hostility + relations_delta.abs() * 0.1).min(2.5);
            }

            for (codename, relations) in impacted {
                self.record(format!(
                    "{} への外交アクション: 関係値 {:.1}%",
                    codename,
                    relations * 100.0
                ));
            }
        }
    }
}
