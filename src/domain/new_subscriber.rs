use super::{subscriber_email::SubscriberEmail, SubscriberName, Subscription};

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<Subscription> for NewSubscriber {
    type Error = String;

    fn try_from(value: Subscription) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { name, email })
    }
}
