--[[
    Healing Station Mod

    Demonstrates ISSUN modding API:
    - Event subscription and handling
    - Commands API for entity manipulation
    - Logging utilities
    - Random number generation

    This mod creates a healing station that heals entities when they enter.
]]

-- Mod state
local heal_count = 0
local heal_amount_min = 10
local heal_amount_max = 30

-- Initialization
function on_init()
    log("Healing Station mod initialized!")
    log(string.format("Heal amount: %d-%d HP", heal_amount_min, heal_amount_max))
end

-- Event handler: Entity entered healing station
function on_entity_enter(event)
    local entity_id = event.entity_id

    log(string.format("Entity %d entered healing station", entity_id))

    -- Calculate random heal amount
    local heal_amount = random_range(heal_amount_min, heal_amount_max)

    log(string.format("Healing entity %d for %.0f HP", entity_id, heal_amount))

    -- Queue command to modify Health component
    -- Note: This is a simplified example - in practice, we'd read current health first
    commands:insert_component(entity_id, "Health", 100)

    heal_count = heal_count + 1
end

-- Event handler: Combat damage dealt
function on_damage_dealt(event)
    local attacker = event.attacker
    local target = event.target
    local damage = event.damage

    log_warn(string.format("Damage event: Entity %d dealt %d damage to Entity %d",
                          attacker, damage, target))

    -- Could trigger healing station activation here
end

-- Query function: Get mod statistics
function get_stats()
    return {
        total_heals = heal_count,
        version = "1.0.0",
        active = true
    }
end

-- Utility function: Set heal parameters
function set_heal_range(min, max)
    heal_amount_min = min
    heal_amount_max = max
    log(string.format("Heal range updated: %d-%d HP", min, max))
end

-- Example of using random numbers
function roll_critical()
    local roll = random()
    local is_critical = roll > 0.9

    if is_critical then
        log("Critical heal! 2x effectiveness")
    end

    return is_critical
end

-- Entry point
log("Healing Station script loaded successfully")
