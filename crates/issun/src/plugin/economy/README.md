# Economy Plugin

Complete economy system with multi-currency support, resource management, and currency exchange.

## Features

### 1. Multi-Currency System
- Define multiple currencies with metadata (name, symbol, decimals)
- Wallet system for tracking currency balances
- Safe arithmetic operations (saturating add/sub)

### 2. Root Resource System
- Define resources with flexible types:
  - **Stock**: Accumulated resources (Gold, Food, Iron)
  - **Flow**: Per-turn generation capacity (GDP, Production)
  - **Abstract**: Abstract values (National Power, Influence)
  - **Hybrid**: Complex multi-aspect resources
- Infinite or finite resource support
- Resource inventory tracking

### 3. Currency Exchange
- Define exchange rates between currencies
- Static or dynamic rate support
- Automatic conversion calculations

### 4. Resource-to-Currency Conversion
- Define conversion rules from resources to currencies
- One resource can convert to multiple currencies
- Automatic resource consumption and currency generation

## Usage Examples

### Example 1: Medieval Fantasy (Stock-based Economy)

```rust
use issun::prelude::*;
use issun::plugin::economy::*;

async fn setup_medieval_economy(game: &mut Game) {
    // Define currencies
    let mut currencies = game.resources.get_mut::<CurrencyDefinitions>().await.unwrap();
    currencies.register(CurrencyDefinition {
        id: CurrencyId::new("gold_coin"),
        name: "Gold Coin".into(),
        symbol: "G".into(),
        decimals: 0,
    });
    currencies.register(CurrencyDefinition {
        id: CurrencyId::new("silver_coin"),
        name: "Silver Coin".into(),
        symbol: "S".into(),
        decimals: 0,
    });
    drop(currencies);

    // Define resources
    let mut resources = game.resources.get_mut::<ResourceDefinitions>().await.unwrap();
    resources.register(
        ResourceDefinition::new(
            "gold_ore",
            "Gold Ore",
            "Raw gold extracted from mines",
            ResourceType::Stock,
        )
        .with_finite()  // Gold ore is finite
    );
    drop(resources);

    // Define conversion: Gold Ore -> Gold Coins
    let mut rules = game.resources.get_mut::<ConversionRules>().await.unwrap();
    rules.register(ConversionRule::new(
        ResourceId::new("gold_ore"),
        CurrencyId::new("gold_coin"),
        10.0,  // 1 Gold Ore = 10 Gold Coins
    ));
    drop(rules);

    // Define exchange: Gold Coins <-> Silver Coins
    let mut rates = game.resources.get_mut::<ExchangeRates>().await.unwrap();
    rates.register(ExchangeRate::new(
        CurrencyId::new("gold_coin"),
        CurrencyId::new("silver_coin"),
        10.0,  // 1 Gold = 10 Silver
    ));
    drop(rates);

    // Initialize inventory with some gold ore
    let mut inventory = game.resources.get_mut::<ResourceInventory>().await.unwrap();
    inventory.insert(ResourceId::new("gold_ore"), 100);
}

async fn mine_and_mint(game: &mut Game) {
    let service = game.services.get::<EconomyService>().unwrap();

    let mut inventory = game.resources.get_mut::<ResourceInventory>().await.unwrap();
    let mut wallet = game.resources.get_mut::<Wallet>().await.unwrap();
    let resource_defs = game.resources.get::<ResourceDefinitions>().await.unwrap();
    let conversion_rules = game.resources.get::<ConversionRules>().await.unwrap();

    // Convert 10 Gold Ore to Gold Coins
    let gold_gained = service.convert_resource_to_currency(
        &mut inventory,
        &mut wallet,
        &resource_defs,
        &conversion_rules,
        &ResourceId::new("gold_ore"),
        &CurrencyId::new("gold_coin"),
        10,
    ).unwrap();

    println!("Minted {} gold coins!", gold_gained.amount());
}
```

### Example 2: Modern Nation Economy (Flow-based)

```rust
use issun::prelude::*;
use issun::plugin::economy::*;

async fn setup_modern_economy(game: &mut Game) {
    // Define currencies
    let mut currencies = game.resources.get_mut::<CurrencyDefinitions>().await.unwrap();
    currencies.register(CurrencyDefinition {
        id: CurrencyId::new("tax_revenue"),
        name: "Tax Revenue".into(),
        symbol: "$".into(),
        decimals: 2,
    });
    currencies.register(CurrencyDefinition {
        id: CurrencyId::new("investment_pool"),
        name: "Investment Pool".into(),
        symbol: "INV".into(),
        decimals: 2,
    });
    drop(currencies);

    // Define Flow-type resource (GDP)
    let mut resources = game.resources.get_mut::<ResourceDefinitions>().await.unwrap();
    resources.register(
        ResourceDefinition::new(
            "gdp",
            "GDP",
            "Gross Domestic Product - Economic capacity",
            ResourceType::Flow { per_turn: true },
        )
        // Infinite by default - GDP is a conceptual measure
    );
    drop(resources);

    // Define conversion: GDP -> Tax Revenue (25% tax rate)
    let mut rules = game.resources.get_mut::<ConversionRules>().await.unwrap();
    rules.register(ConversionRule::new(
        ResourceId::new("gdp"),
        CurrencyId::new("tax_revenue"),
        0.25,  // 25% of GDP becomes tax revenue
    ));
    rules.register(ConversionRule::new(
        ResourceId::new("gdp"),
        CurrencyId::new("investment_pool"),
        0.15,  // 15% of GDP becomes investment capital
    ));
    drop(rules);

    // Initialize GDP capacity
    let mut inventory = game.resources.get_mut::<ResourceInventory>().await.unwrap();
    inventory.insert(ResourceId::new("gdp"), 10000);  // GDP of 10,000
}

async fn collect_taxes(game: &mut Game) {
    let service = game.services.get::<EconomyService>().unwrap();

    let mut inventory = game.resources.get_mut::<ResourceInventory>().await.unwrap();
    let mut wallet = game.resources.get_mut::<Wallet>().await.unwrap();
    let resource_defs = game.resources.get::<ResourceDefinitions>().await.unwrap();
    let conversion_rules = game.resources.get::<ConversionRules>().await.unwrap();

    let gdp = service.resource_quantity(&inventory, &ResourceId::new("gdp"));

    // Convert GDP to tax revenue
    let tax_collected = service.convert_resource_to_currency(
        &mut inventory,
        &mut wallet,
        &resource_defs,
        &conversion_rules,
        &ResourceId::new("gdp"),
        &CurrencyId::new("tax_revenue"),
        gdp,
    ).unwrap();

    println!("Collected {} in taxes from GDP of {}", tax_collected.amount(), gdp);
}
```

### Example 3: Currency Exchange

```rust
use issun::prelude::*;
use issun::plugin::economy::*;

async fn exchange_currencies(game: &mut Game) {
    let service = game.services.get::<EconomyService>().unwrap();

    let mut wallet = game.resources.get_mut::<Wallet>().await.unwrap();
    let rates = game.resources.get::<ExchangeRates>().await.unwrap();

    // Player has 100 USD and wants to exchange to JPY
    wallet.insert(CurrencyId::new("usd"), Currency::new(100));

    let jpy_received = service.exchange(
        &mut wallet,
        &rates,
        &CurrencyId::new("usd"),
        &CurrencyId::new("jpy"),
        Currency::new(50),  // Exchange 50 USD
    ).unwrap();

    println!("Exchanged 50 USD for {} JPY", jpy_received.amount());
}
```

## Architecture

Following the [Plugin Design Principles](../../docs/architecture/plugin-design-principles.md):

### Resources (ReadOnly)
- `CurrencyDefinitions`: Currency metadata registry
- `ResourceDefinitions`: Resource metadata registry
- `ExchangeRates`: Currency exchange rate registry
- `ConversionRules`: Resource-to-currency conversion rules
- `EconomyConfig`: Global economy configuration

### Runtime State (Mutable)
- `Wallet`: Currency balances (`Store<CurrencyId, Currency>`)
- `ResourceInventory`: Resource quantities (`Store<ResourceId, i64>`)

### Service (Stateless)
- `EconomyService`: Pure functions for:
  - Currency operations (deposit, withdraw, transfer)
  - Currency exchange
  - Resource operations (add, consume)
  - Resource-to-currency conversion

## Design Philosophy

### Stock vs Flow Resources

**Stock Resources** represent accumulated quantities:
- Gold, Food, Iron
- Consumed when used
- Can be stored indefinitely
- Finite or infinite

**Flow Resources** represent generation capacity:
- GDP, Production Capacity
- Represent "per turn" generation
- Used as a basis for currency generation
- Typically infinite (conceptual measures)

**Abstract Resources** represent intangible concepts:
- National Power, Influence, Prestige
- May or may not decrease when used
- Game-specific interpretation

**Hybrid Resources** combine multiple aspects:
- Industrial Capacity (Stock count + Flow effect)
- Technology Level (Abstract value + Flow multiplier)

### Infinite vs Finite Resources

- **Infinite Resources**: No consumption check, represents unlimited capacity
  - Default behavior
  - Good for: GDP, abstract concepts, Flow-type resources

- **Finite Resources**: Consumption requires availability check
  - Must explicitly set with `.with_finite()`
  - Good for: Gold, Food, physical Stock-type resources

## Error Handling

The `EconomyService` returns `EconomyResult<T>` which can contain:
- `EconomyError::InsufficientFunds`: Not enough currency for exchange
- `EconomyError::InsufficientResource`: Not enough resource for conversion (finite only)
- `EconomyError::ExchangeRateNotFound`: No rate defined for currency pair
- `EconomyError::ConversionRuleNotFound`: No conversion rule for resource->currency

## Future Enhancements

Potential additions (not yet implemented):
- Events for exchange/conversion operations
- System for automatic per-turn resource generation (Flow types)
- Hooks for custom exchange rate calculations
- Market dynamics for dynamic exchange rates
- Transaction history tracking
- Multi-wallet support (per-entity wallets)
