use bot_poketrue_rust::game::catalog::Catalog;
use bot_poketrue_rust::game::types::PokeTrueState;
use serde_json::json;

#[test]
fn test_poketrue_catalog_loaded() {
    let hunts = Catalog::get_all_hunts();
    assert!(!hunts.is_empty());
    assert!(hunts.len() >= 5);
}

#[test]
fn test_poketrue_route_solver() {
    let catalog = Catalog::default();
    let hunt = catalog.select_best_hunt(20);
    assert!(!hunt.id.is_empty());
}

#[test]
fn test_poketrue_state_deserialization() {
    let raw = json!({
        "estado": {
            "nome": "Ash Ketchum",
            "level": 45,
            "exp": 125000,
            "gold": 54000,
            "time": [
                {
                    "uid": "p-1",
                    "id": 25,
                    "name": "Pikachu",
                    "level": 50,
                    "hp": 120,
                    "maxHp": 120,
                    "heldItem": "light_ball"
                }
            ]
        }
    });

    let st: PokeTrueState = serde_json::from_value(raw.get("estado").unwrap().clone()).unwrap();
    assert_eq!(st.level, 45);
    assert_eq!(st.team.len(), 1);
    assert_eq!(st.team[0].name, "Pikachu");
}
