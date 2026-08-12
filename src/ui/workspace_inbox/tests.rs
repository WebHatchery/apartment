use super::*;

#[test]
fn every_mail_type_owns_a_unique_atlas_cell() {
    let types = [
        MailType::TenantLetter { tenant_id: 1 },
        MailType::CityNotice,
        MailType::Financial,
        MailType::Advertisement,
        MailType::News,
        MailType::Personal,
        MailType::Official,
    ];
    let mut cells = types.map(|mail_type| mail_icon_tile(&mail_type)).to_vec();
    cells.sort_by(|left, right| left.partial_cmp(right).unwrap());
    cells.dedup();
    assert_eq!(cells.len(), 7);
}
