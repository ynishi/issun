//! Systems for economy plugin

use super::events::*;
use super::resources::{ConversionRules, ResourceDefinitions};
use super::service::EconomyService;
use super::state::{ResourceInventory, Wallet};
use super::types::ResourceType;
use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;

/// System for economy orchestration
///
/// Responsibilities:
/// - Process command events (exchange, conversion requests)
/// - Generate currency from Flow resources automatically
#[derive(Default)]
pub struct EconomySystem;

#[async_trait::async_trait]
impl System for EconomySystem {
    fn name(&self) -> &'static str {
        "economy_system"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl EconomySystem {
    /// Process economy command events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let economy_service = match services.get("economy_service") {
            Some(service) => match service.as_any().downcast_ref::<EconomyService>() {
                Some(s) => s,
                None => return,
            },
            None => return,
        };

        // Collect command events
        let (exchange_requests, conversion_requests, add_requests, consume_requests) = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let exchange = bus
                    .reader::<CurrencyExchangeRequested>()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                let conversion = bus
                    .reader::<ResourceConversionRequested>()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                let add = bus
                    .reader::<ResourceAddRequested>()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                let consume = bus
                    .reader::<ResourceConsumeRequested>()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                (exchange, conversion, add, consume)
            } else {
                return;
            }
        };

        // Process currency exchange requests
        for request in exchange_requests {
            self.process_exchange_request(request, economy_service, resources)
                .await;
        }

        // Process resource conversion requests
        for request in conversion_requests {
            self.process_conversion_request(request, economy_service, resources)
                .await;
        }

        // Process resource add requests
        for request in add_requests {
            self.process_add_request(request, economy_service, resources)
                .await;
        }

        // Process resource consume requests
        for request in consume_requests {
            self.process_consume_request(request, economy_service, resources)
                .await;
        }
    }

    /// Process currency exchange request
    async fn process_exchange_request(
        &mut self,
        request: CurrencyExchangeRequested,
        service: &EconomyService,
        resources: &mut ResourceContext,
    ) {
        let mut wallet = match resources.get_mut::<Wallet>().await {
            Some(w) => w,
            None => return,
        };

        let exchange_rates = match resources.get::<super::resources::ExchangeRates>().await {
            Some(r) => r,
            None => {
                drop(wallet);
                self.emit_exchange_failed(
                    request,
                    "Exchange rates not available".to_string(),
                    resources,
                )
                .await;
                return;
            }
        };

        match service.exchange(
            &mut wallet,
            &exchange_rates,
            &request.from_currency,
            &request.to_currency,
            request.from_amount,
        ) {
            Ok(to_amount) => {
                drop(wallet);
                drop(exchange_rates);
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(CurrencyExchanged {
                        from_currency: request.from_currency,
                        to_currency: request.to_currency,
                        from_amount: request.from_amount,
                        to_amount,
                    });
                }
            }
            Err(e) => {
                drop(wallet);
                drop(exchange_rates);
                self.emit_exchange_failed(request, format!("{:?}", e), resources)
                    .await;
            }
        }
    }

    /// Process resource conversion request
    async fn process_conversion_request(
        &mut self,
        request: ResourceConversionRequested,
        service: &EconomyService,
        resources: &mut ResourceContext,
    ) {
        let mut inventory = match resources.get_mut::<ResourceInventory>().await {
            Some(i) => i,
            None => return,
        };

        let mut wallet = match resources.get_mut::<Wallet>().await {
            Some(w) => w,
            None => return,
        };

        let resource_defs = match resources.get::<ResourceDefinitions>().await {
            Some(d) => d,
            None => {
                drop(inventory);
                drop(wallet);
                self.emit_conversion_failed(
                    request,
                    "Resource definitions not available".to_string(),
                    resources,
                )
                .await;
                return;
            }
        };

        let conversion_rules = match resources.get::<ConversionRules>().await {
            Some(r) => r,
            None => {
                drop(inventory);
                drop(wallet);
                drop(resource_defs);
                self.emit_conversion_failed(
                    request,
                    "Conversion rules not available".to_string(),
                    resources,
                )
                .await;
                return;
            }
        };

        match service.convert_resource_to_currency(
            &mut inventory,
            &mut wallet,
            &resource_defs,
            &conversion_rules,
            &request.resource_id,
            &request.currency_id,
            request.resource_amount,
        ) {
            Ok(currency_amount) => {
                drop(inventory);
                drop(wallet);
                drop(resource_defs);
                drop(conversion_rules);
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(ResourceConverted {
                        resource_id: request.resource_id,
                        currency_id: request.currency_id,
                        resource_amount: request.resource_amount,
                        currency_amount,
                    });
                }
            }
            Err(e) => {
                drop(inventory);
                drop(wallet);
                drop(resource_defs);
                drop(conversion_rules);
                self.emit_conversion_failed(request, format!("{:?}", e), resources)
                    .await;
            }
        }
    }

    /// Process resource add request
    async fn process_add_request(
        &mut self,
        request: ResourceAddRequested,
        service: &EconomyService,
        resources: &mut ResourceContext,
    ) {
        let mut inventory = match resources.get_mut::<ResourceInventory>().await {
            Some(i) => i,
            None => return,
        };

        let resource_defs = match resources.get::<ResourceDefinitions>().await {
            Some(d) => d,
            None => {
                drop(inventory);
                return;
            }
        };

        if service
            .add_resource(
                &mut inventory,
                &resource_defs,
                &request.resource_id,
                request.amount,
            )
            .is_ok()
        {
            let new_total = service.resource_quantity(&inventory, &request.resource_id);
            drop(inventory);
            drop(resource_defs);
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResourceAdded {
                    resource_id: request.resource_id,
                    amount: request.amount,
                    new_total,
                });
            }
        }
    }

    /// Process resource consume request
    async fn process_consume_request(
        &mut self,
        request: ResourceConsumeRequested,
        service: &EconomyService,
        resources: &mut ResourceContext,
    ) {
        let mut inventory = match resources.get_mut::<ResourceInventory>().await {
            Some(i) => i,
            None => return,
        };

        let resource_defs = match resources.get::<ResourceDefinitions>().await {
            Some(d) => d,
            None => {
                drop(inventory);
                return;
            }
        };

        match service.consume_resource(
            &mut inventory,
            &resource_defs,
            &request.resource_id,
            request.amount,
        ) {
            Ok(_) => {
                let remaining = service.resource_quantity(&inventory, &request.resource_id);
                drop(inventory);
                drop(resource_defs);
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(ResourceConsumed {
                        resource_id: request.resource_id,
                        amount: request.amount,
                        remaining,
                    });
                }
            }
            Err(e) => {
                drop(inventory);
                drop(resource_defs);
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(ResourceConsumeFailed {
                        resource_id: request.resource_id,
                        amount: request.amount,
                        reason: format!("{:?}", e),
                    });
                }
            }
        }
    }

    /// Generate currency from Flow resources automatically
    ///
    /// This should be called once per turn/period. It checks all Flow resources
    /// with `per_turn: true` and automatically converts them to currency according
    /// to conversion rules.
    pub async fn generate_flow_resources(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let economy_service = match services.get("economy_service") {
            Some(service) => match service.as_any().downcast_ref::<EconomyService>() {
                Some(s) => s,
                None => return,
            },
            None => return,
        };

        let inventory = match resources.get::<ResourceInventory>().await {
            Some(i) => i,
            None => return,
        };

        let resource_defs = match resources.get::<ResourceDefinitions>().await {
            Some(d) => d,
            None => {
                drop(inventory);
                return;
            }
        };

        let conversion_rules = match resources.get::<ConversionRules>().await {
            Some(r) => r,
            None => {
                drop(inventory);
                drop(resource_defs);
                return;
            }
        };

        // Find all Flow resources with per_turn = true
        let mut flow_generations = Vec::new();

        for def in resource_defs.all() {
            if let ResourceType::Flow { per_turn: true } = def.resource_type {
                let capacity = economy_service.resource_quantity(&inventory, &def.id);

                if capacity > 0 {
                    // Get all conversion rules for this resource
                    if let Some(rules) = conversion_rules.get(&def.id) {
                        for rule in rules {
                            let currency_amount = rule.convert(capacity);
                            flow_generations.push((
                                def.id.clone(),
                                rule.currency.clone(),
                                capacity,
                                currency_amount,
                            ));
                        }
                    }
                }
            }
        }

        drop(inventory);
        drop(resource_defs);
        drop(conversion_rules);

        // Apply all flow generations
        let mut wallet = match resources.get_mut::<Wallet>().await {
            Some(w) => w,
            None => return,
        };

        for (resource_id, currency_id, capacity, amount) in flow_generations {
            economy_service.deposit(&mut wallet, &currency_id, amount);

            // Emit event
            drop(wallet);
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(FlowResourceGenerated {
                    resource_id,
                    currency_id: currency_id.clone(),
                    resource_capacity: capacity,
                    currency_generated: amount,
                });
            }
            wallet = match resources.get_mut::<Wallet>().await {
                Some(w) => w,
                None => return,
            };
        }
    }

    async fn emit_exchange_failed(
        &self,
        request: CurrencyExchangeRequested,
        reason: String,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(CurrencyExchangeFailed {
                from_currency: request.from_currency,
                to_currency: request.to_currency,
                from_amount: request.from_amount,
                reason,
            });
        }
    }

    async fn emit_conversion_failed(
        &self,
        request: ResourceConversionRequested,
        reason: String,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(ResourceConversionFailed {
                resource_id: request.resource_id,
                currency_id: request.currency_id,
                resource_amount: request.resource_amount,
                reason,
            });
        }
    }
}
