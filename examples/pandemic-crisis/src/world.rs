//! World setup - City network and travel routes

use bevy::prelude::*;
use issun_bevy::plugins::contagion::*;

/// City data for world setup
pub struct CityData {
    pub id: &'static str,
    pub name: &'static str,
    pub population: usize,
    pub resistance: f32,
}

/// Travel route data
pub struct RouteData {
    pub id: &'static str,
    pub from: &'static str,
    pub to: &'static str,
    pub transmission_rate: f32,
}

/// World configuration
pub const CITIES: &[CityData] = &[
    CityData {
        id: "tokyo",
        name: "Tokyo",
        population: 14_000_000,
        resistance: 0.7,
    },
    CityData {
        id: "newyork",
        name: "New York",
        population: 8_300_000,
        resistance: 0.6,
    },
    CityData {
        id: "mumbai",
        name: "Mumbai",
        population: 12_000_000,
        resistance: 0.4,
    },
    CityData {
        id: "lagos",
        name: "Lagos",
        population: 14_000_000,
        resistance: 0.2,
    },
    CityData {
        id: "saopaulo",
        name: "São Paulo",
        population: 12_000_000,
        resistance: 0.4,
    },
    CityData {
        id: "london",
        name: "London",
        population: 9_000_000,
        resistance: 0.6,
    },
    CityData {
        id: "beijing",
        name: "Beijing",
        population: 21_000_000,
        resistance: 0.5,
    },
    CityData {
        id: "cairo",
        name: "Cairo",
        population: 10_000_000,
        resistance: 0.3,
    },
];

/// Travel routes connecting cities
pub const ROUTES: &[RouteData] = &[
    // Major hubs
    RouteData {
        id: "route_tokyo_newyork",
        from: "tokyo",
        to: "newyork",
        transmission_rate: 0.7,
    },
    RouteData {
        id: "route_newyork_london",
        from: "newyork",
        to: "london",
        transmission_rate: 0.8,
    },
    RouteData {
        id: "route_london_cairo",
        from: "london",
        to: "cairo",
        transmission_rate: 0.6,
    },
    RouteData {
        id: "route_cairo_mumbai",
        from: "cairo",
        to: "mumbai",
        transmission_rate: 0.5,
    },
    RouteData {
        id: "route_mumbai_beijing",
        from: "mumbai",
        to: "beijing",
        transmission_rate: 0.6,
    },
    RouteData {
        id: "route_beijing_tokyo",
        from: "beijing",
        to: "tokyo",
        transmission_rate: 0.8,
    },
    // Africa-South America
    RouteData {
        id: "route_lagos_saopaulo",
        from: "lagos",
        to: "saopaulo",
        transmission_rate: 0.4,
    },
    RouteData {
        id: "route_saopaulo_newyork",
        from: "saopaulo",
        to: "newyork",
        transmission_rate: 0.6,
    },
    // Africa connections
    RouteData {
        id: "route_cairo_lagos",
        from: "cairo",
        to: "lagos",
        transmission_rate: 0.5,
    },
    // Asia-Europe
    RouteData {
        id: "route_beijing_london",
        from: "beijing",
        to: "london",
        transmission_rate: 0.7,
    },
    // Mumbai-Lagos trade
    RouteData {
        id: "route_mumbai_lagos",
        from: "mumbai",
        to: "lagos",
        transmission_rate: 0.3,
    },
    // São Paulo-London
    RouteData {
        id: "route_saopaulo_london",
        from: "saopaulo",
        to: "london",
        transmission_rate: 0.5,
    },
];

/// Setup world system - spawn cities and routes
pub fn setup_world(
    mut commands: Commands,
    mut node_registry: ResMut<NodeRegistry>,
    mut edge_registry: ResMut<EdgeRegistry>,
) {
    info!("Setting up world with {} cities and {} routes", CITIES.len(), ROUTES.len());

    // Spawn cities
    for city_data in CITIES {
        let entity = commands
            .spawn(ContagionNode::new(
                city_data.id,
                NodeType::City,
                city_data.population,
            )
            .with_resistance(city_data.resistance))
            .id();

        node_registry.register(city_data.id, entity);
        info!("  City: {} (pop: {}, resistance: {})",
            city_data.name, city_data.population, city_data.resistance);
    }

    // Spawn routes
    for route_data in ROUTES {
        let from_entity = node_registry.get(route_data.from)
            .expect("From city should exist");
        let to_entity = node_registry.get(route_data.to)
            .expect("To city should exist");

        let edge_entity = commands
            .spawn(PropagationEdge::new(
                route_data.id,
                from_entity,
                to_entity,
                route_data.transmission_rate,
            ))
            .id();

        edge_registry.register(route_data.id, edge_entity);
    }

    info!("World setup complete!");
}

/// Get city name by ID
pub fn get_city_name(city_id: &str) -> &'static str {
    CITIES
        .iter()
        .find(|c| c.id == city_id)
        .map(|c| c.name)
        .unwrap_or("Unknown")
}

/// Get total world population
pub fn get_total_population() -> usize {
    CITIES.iter().map(|c| c.population).sum()
}

/// Get city by ID
pub fn get_city_data(city_id: &str) -> Option<&'static CityData> {
    CITIES.iter().find(|c| c.id == city_id)
}
