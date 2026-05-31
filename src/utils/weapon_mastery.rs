pub fn has_mastery(weapon_name: &str) -> bool {
    let name = weapon_name.to_lowercase();
    matches!(
        name.as_str(),
        "battleaxe"
            | "blowgun"
            | "club"
            | "dagger"
            | "dart"
            | "flail"
            | "glaive"
            | "greataxe"
            | "greatsword"
            | "halberd"
            | "handaxe"
            | "hand crossbow"
            | "heavy crossbow"
            | "javelin"
            | "light crossbow"
            | "light hammer"
            | "longbow"
            | "longsword"
            | "mace"
            | "maul"
            | "morningstar"
            | "musket"
            | "pike"
            | "pistol"
            | "quarterstaff"
            | "scimitar"
            | "shortbow"
            | "shortsword"
            | "sickle"
            | "sling"
            | "spear"
            | "trident"
            | "warhammer"
            | "war pick"
            | "whip"
    )
}

pub fn get_mastery_property(weapon_name: &str) -> &'static str {
    match weapon_name.to_lowercase().as_str() {
        "battleaxe" => "Topple",
        "blowgun" => "Vex",
        "club" => "Slow",
        "dagger" => "Nick",
        "dart" => "Vex",
        "flail" => "Sap",
        "glaive" => "Graze",
        "greataxe" => "Cleave",
        "greatsword" => "Graze",
        "halberd" => "Cleave",
        "handaxe" => "Vex",
        "hand crossbow" => "Vex",
        "heavy crossbow" => "Push",
        "javelin" => "Slow",
        "light crossbow" => "Slow",
        "light hammer" => "Nick",
        "longbow" => "Slow",
        "longsword" => "Sap",
        "mace" => "Sap",
        "maul" => "Topple",
        "morningstar" => "Sap",
        "musket" => "Slow",
        "pike" => "Push",
        "pistol" => "Vex",
        "quarterstaff" => "Topple",
        "scimitar" => "Nick",
        "shortbow" => "Vex",
        "shortsword" => "Vex",
        "sickle" => "Nick",
        "sling" => "Slow",
        "spear" => "Sap",
        "trident" => "Topple",
        "warhammer" => "Topple",
        "war pick" => "Sap",
        "whip" => "Slow",
        _ => "—",
    }
}

pub fn get_mastery_description(mastery_name: &str) -> &'static str {
    match mastery_name.to_lowercase().as_str() {
        "cleave" => "If you hit a creature with a melee attack roll using this weapon, you can make a second attack roll against a different creature that is within 5 feet of the first component and within your reach. On a hit, the second creature takes the weapon's damage, but don't add your ability modifier to that damage unless that modifier is negative.",
        "graze" => "If your attack roll with this weapon misses a creature, you can deal damage to that creature equal to the ability modifier you used to make the attack roll. This damage is the same type dealt by the weapon, and the damage can be increased only by increasing the ability modifier.",
        "nick" => "When you make an extra attack with the Light property, you can make it as part of the Attack action instead of as a Bonus Action. You can still make only one such extra attack per turn.",
        "push" => "If you hit a creature with this weapon, you can push the creature up to 10 feet straight away from yourself if it is no more than one size larger than you.",
        "sap" => "If you hit a creature with this weapon, that creature has Disadvantage on its next attack roll before the start of your next turn.",
        "slow" => "If you hit a creature with this weapon and deal damage, you can reduce the creature's Speed by 10 feet until the start of your next turn. If you hit the creature again before then, its Speed isn't reduced further by this property.",
        "topple" => "If you hit a creature with this weapon, you can force the creature to make a Constitution saving throw with a DC equal to 8 + your Proficiency Bonus + the ability modifier used to make the attack roll. On a failed save, the creature has the Prone condition.",
        "vex" => "If you hit a creature with this weapon and deal damage, you have Advantage on your next attack roll against that creature before the end of your next turn.",
        _ => "No description available.",
    }
}
