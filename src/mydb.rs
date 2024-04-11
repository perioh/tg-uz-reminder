use core::hash::Hash;
use dashmap::DashMap;
use database::Database;
use pdf_parser::TicketData;
use rclite::Arc;
use std::collections::HashSet;
use tracing::trace;

#[derive(Clone)]
pub struct MyDb<K, V>(Arc<DashMap<K, V>>);

impl<K: Eq + Hash, V> MyDb<K, V> {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }
}

impl Database for MyDb<telegram::ChatId, HashSet<TicketData>> {
    type Error = DatabaseError;
    type Ticket = TicketData;
    type User = telegram::ChatId;

    fn insert_ticket_data(
        &self,
        user_id: Self::User,
        ticket_data: Self::Ticket,
    ) -> Result<(), Self::Error> {
        if let Some(mut data) = self.0.get_mut(&user_id) {
            let x = data.value_mut();
            trace!(?ticket_data, %user_id, "inserting old hashset");
            x.insert(ticket_data);
        } else {
            let mut hashset = HashSet::new();
            trace!(?ticket_data, %user_id, "inserting to new hashset");
            hashset.insert(ticket_data);

            self.0.insert(user_id, hashset);
        }
        Ok(())
    }

    fn retrieve_user_trains(&self, user_id: Self::User) -> impl Iterator<Item = TicketData> {
        let user_trains = self
            .0
            .get(&user_id)
            .map(|data| data.value().iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        user_trains.into_iter()
    }

    fn remove_user_train(
        &self,
        user_id: Self::User,
        train: Self::Ticket,
    ) -> Result<(), Self::Error> {
        let Some(mut data) = self.0.get_mut(&user_id) else {
            return Err(DatabaseError::UserNotExisting);
        };
        trace!(?train, %user_id, "removing from db");

        data.value_mut().remove(&train);
        Ok(())
    }

    fn users(&self) -> impl Iterator<Item = (Self::User, impl Iterator<Item = TicketData>)> {
        self.0.iter().map(|user| {
            let (key, value) = user.pair();
            let key = key.to_owned();
            let value = value.to_owned();
            (key, value.into_iter())
        })
    }

    // fn find_users_by_train(
    //     &self,
    //     number: &str,
    // ) -> impl Iterator<Item = (Self::User, Self::DepartureTime)> {
    //     let z = self.0.iter().filter_map(move |user| {
    //         let (key, value) = user.pair();

    //         if let Some(value) = value.iter().find(|ticket| ticket.train_number == number) {
    //             Some((key.to_owned(), value.departure_datetime))
    //         } else {
    //             None
    //         }
    //     });
    //     z
    //     // todo!()
    // }

    // fn filter<P>(&self, p: P) -> impl Iterator<Item = Self::Ticket>
    // where
    //     P: FnMut(&Self::Ticket) -> bool,
    // {
    //     self.0.iter().map(|x| x.to_owned()).flatten().filter(p)

    //     // for x in self.0.iter() {
    //     //     x.iter().cloned().filter(f).collect::<Vec<_>>();
    //     // }

    //     // let x = self
    //     // .0
    //     // .iter()
    //     // .map(|x| x.iter().cloned().filter(f).collect::<Vec<_>>())
    //     // .flatten();
    //     // todo!()
    // }
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("User does not exist")]
    UserNotExisting,
}
