use anyhow::Result;
use sqlx::{prelude::FromRow, query, query_as, Sqlite};

use crate::utils::sqlite::RoswaalSqliteTransaction;

use super::{coordinate::LocationCoordinate2D, location::RoswaalLocation, name::RoswaalLocationName};

impl <'a> RoswaalSqliteTransaction <'a> {
    pub async fn save_locations(&mut self, locations: &Vec<RoswaalLocation>) -> Result<()> {
        let statements = locations.iter()
            .map(|_| {
                "INSERT OR REPLACE INTO Locations (latitude, longitude, name) VALUES (?, ?, ?);"
            })
            .collect::<Vec<&str>>()
            .join("\n");
        let mut bulk_insert_query = query::<Sqlite>(&statements);
        for location in locations.iter() {
            bulk_insert_query = bulk_insert_query.bind(location.coordinate().latitude())
                .bind(location.coordinate().longitude())
                .bind(&location.name().raw_value)
        }
        bulk_insert_query.execute(self.connection()).await?;
        Ok(())
    }

    pub async fn locations_in_alphabetical_order(&mut self) -> Result<Vec<RoswaalLocation>> {
        let locations = query_as::<Sqlite, SqliteLocation>(
            "SELECT * FROM Locations ORDER BY name"
        )
        .fetch_all(self.connection())
        .await?
        .iter()
        .map(|l| {
            RoswaalLocation::new(
                RoswaalLocationName { raw_value: l.name.clone() },
                LocationCoordinate2D { latitude: l.latitude, longitude: l.longitude }
            )
        })
        .collect();
        Ok(locations)
    }
}

#[derive(FromRow, Clone)]
struct SqliteLocation {
    latitude: f32,
    longitude: f32,
    name: String
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{location::{coordinate::LocationCoordinate2D, location::RoswaalLocation, name::RoswaalLocationName}, utils::sqlite::{RoswaalSqlite, RoswaalSqliteTransaction}};

    #[tokio::test]
    async fn test_add_and_load_locations_no_prior_locations() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let locations = vec![
            RoswaalLocation::new(
                RoswaalLocationName::from_str("Antarctica").unwrap(),
                LocationCoordinate2D::try_new(32.29873932, 122.3939839).unwrap()
            ),
            RoswaalLocation::new(
                RoswaalLocationName::from_str("New York").unwrap(),
                LocationCoordinate2D::try_new(45.0, 45.0).unwrap()
            )
        ];
        _ = transaction.save_locations(&locations).await;
        let saved_locations = transaction.locations_in_alphabetical_order().await.unwrap();
        assert_eq!(locations, saved_locations)
    }

    #[tokio::test]
    async fn test_add_and_load_locations_upserts_prior_locations() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        let mut locations = vec![
            RoswaalLocation::new(
                RoswaalLocationName::from_str("Antarctica").unwrap(),
                LocationCoordinate2D::try_new(32.29873932, 122.3939839).unwrap()
            ),
            RoswaalLocation::new(
                RoswaalLocationName::from_str("New York").unwrap(),
                LocationCoordinate2D::try_new(45.0, 45.0).unwrap()
            )
        ];
        _ = transaction.save_locations(&locations).await;
        locations = vec![
            RoswaalLocation::new(
                RoswaalLocationName::from_str("Antarctica").unwrap(),
                LocationCoordinate2D::try_new(50.0, 50.0).unwrap()
            ),
            RoswaalLocation::new(
                RoswaalLocationName::from_str("Oakland").unwrap(),
                LocationCoordinate2D::try_new(45.0, 45.0).unwrap()
            )
        ];
        _ = transaction.save_locations(&locations).await;
        let saved_locations = transaction.locations_in_alphabetical_order().await.unwrap();
        assert_eq!(
            saved_locations,
            vec![
                RoswaalLocation::new(
                    RoswaalLocationName::from_str("Antarctica").unwrap(),
                    LocationCoordinate2D::try_new(50.0, 50.0).unwrap()
                ),
                RoswaalLocation::new(
                    RoswaalLocationName::from_str("New York").unwrap(),
                    LocationCoordinate2D::try_new(45.0, 45.0).unwrap()
                ),
                RoswaalLocation::new(
                    RoswaalLocationName::from_str("Oakland").unwrap(),
                    LocationCoordinate2D::try_new(45.0, 45.0).unwrap()
                )
            ]
        )
    }
}
