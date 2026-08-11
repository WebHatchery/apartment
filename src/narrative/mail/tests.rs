use super::*;

#[test]
fn test_mailbox_basics() {
    let mut mailbox = Mailbox::new();
    mailbox.receive(MailItem::news_clipping(0, 1, "Test", "Body"));

    assert_eq!(mailbox.unread_count(), 1);

    let id = mailbox.items[0].id;
    mailbox.mark_read(id);

    assert_eq!(mailbox.unread_count(), 0);
}

#[test]
fn test_mail_priority() {
    assert!(MailType::CityNotice.priority() > MailType::Advertisement.priority());
}
