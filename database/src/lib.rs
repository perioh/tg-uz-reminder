pub trait Database {
    type User;
    type Ticket;
    type Error;

    fn insert_ticket_data(
        &self,
        user: Self::User,
        ticket_data: Self::Ticket,
    ) -> Result<(), Self::Error>;

    fn retrieve_user_trains(&self, user_id: Self::User) -> impl Iterator<Item = Self::Ticket>;
    fn users(&self) -> impl Iterator<Item = (Self::User, impl Iterator<Item = Self::Ticket>)>;
    fn remove_user_train(
        &self,
        user_id: Self::User,
        train: Self::Ticket,
    ) -> Result<(), Self::Error>;
}
