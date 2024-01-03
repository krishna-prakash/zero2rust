mod new_subscriber;
mod subscriber_email;
mod subscriber_name;

pub use new_subscriber::NewSubscriber;
pub use subscriber_email::SubscriberEmail;
pub use subscriber_name::SubscriberName;

#[derive(serde::Deserialize)]
pub struct Subscription {
    pub email: String,
    pub name: String,
}
