/// Parse weapon property code from backend format (e.g., "H|XPHB" -> "H")
pub fn parse_property_code(prop: &str) -> &str {
    prop.split('|').next().unwrap_or("")
}

/// Get human-readable property name
pub fn property_name(code: &str) -> String {
    match code.to_uppercase().as_str() {
        "A" => "Ammunition".to_string(),
        "F" => "Finesse".to_string(),
        "H" => "Heavy".to_string(),
        "L" => "Light".to_string(),
        "LD" => "Loading".to_string(),
        "R" => "Reach".to_string(),
        "RLD" => "Reload".to_string(),
        "T" => "Thrown".to_string(),
        "V" => "Versatile".to_string(),
        "2H" => "Two-Handed".to_string(),
        _ => code.to_string(),
    }
}

/// Get full property description
pub fn property_description(code: &str) -> &'static str {
    match code.to_uppercase().as_str() {
        "A" => "You can use a weapon that has the Ammunition property to make a ranged attack only if you have ammunition to fire from the weapon. Each time you attack with the weapon, you expend one piece of ammunition. Drawing the ammunition from a quiver, case, or other container is part of the attack (you need a free hand to load a one-handed weapon). At the end of the battle, you can recover half your expended ammunition by taking a minute to search the battlefield.",
        "F" => "When making an attack with a Finesse weapon, you use your choice of your Strength or Dexterity modifier for the attack and damage rolls. You must use the same modifier for both rolls.",
        "H" => "You have Disadvantage on attack rolls with a Heavy weapon if it's a Melee weapon and your Strength score isn't at least 13 or if it's a Ranged weapon and your Dexterity score isn't at least 13.",
        "L" => "A Light weapon is small and easy to handle, making it ideal for use when fighting with two weapons.",
        "LD" => "Because of the time required to load this weapon, you can fire only one piece of ammunition from it when you use an action, Bonus Action, or Reaction to fire it, regardless of the number of attacks you can normally make.",
        "R" => "A Reach weapon adds 5 feet to your reach when you attack with it, as well as when determining your reach for Opportunity Attacks with it.",
        "RLD" => "A weapon with the Reload property requires you to reload after each shot. It holds ammunition equal to the number in its property (for example, Reload 5), and you can reload it as part of your attack. Some reload weapons have a longer reload time (given in the property), meaning you must spend an action to reload it. If you expend all of its ammunition without reloading, the weapon becomes unusable until you reload it.",
        "T" => "If a weapon has the Thrown property, you can throw the weapon to make a ranged attack. If the weapon is a Melee weapon, you use the same ability modifier for that ranged attack that you would use for a melee attack with the weapon. For example, if you throw a Handaxe, you use your Strength, but if you throw a Dagger, you can use either your Strength or Dexterity, since the Dagger has the Finesse property.",
        "V" => "This weapon can be used with one or two hands. A damage value in parentheses appears with the property — the damage when the weapon is used with two hands to make a melee attack.",
        "2H" => "This weapon requires two hands when you attack with it.",
        _ => "No description available.",
    }
}

/// Get all property descriptions for a weapon as formatted lines
pub fn format_all_properties(properties: &[String]) -> Vec<String> {
    let mut lines = Vec::new();

    for prop in properties {
        let code = parse_property_code(prop);
        let name = property_name(code);
        let desc = property_description(code);

        lines.push(format!("{}.", name));
        lines.push(format!("  {}", desc));
        lines.push(String::new());
    }

    lines
}
