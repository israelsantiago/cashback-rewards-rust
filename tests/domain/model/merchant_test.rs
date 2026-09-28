use cashback_rewards_rust::domain::model::Merchant;

#[test]
fn merchants_carry_a_partner_flag() {
    assert!(
        Merchant {
            name: "GreenGrocer".into(),
            partner: true
        }
        .partner
    );
    assert!(
        !Merchant {
            name: "Corner Cafe".into(),
            partner: false
        }
        .partner
    );
}
