use anyhow::Result;
use sqlx::{prelude::FromRow, query, query_as, Sqlite};

use crate::{utils::sqlite::RoswaalSqlite, with_transaction};

use super::{coordinate::LocationCoordinate2D, location::RoswaalLocation, name::RoswaalLocationName};

pub struct RoswaalLocationsStorage {
    sqlite: RoswaalSqlite
}

impl RoswaalLocationsStorage {
    pub fn new(sqlite: RoswaalSqlite) -> Self {
        Self { sqlite }
    }
}

const UPSERT_QUERY: &str = "
INSERT OR REPLACE INTO Locations (latitude, longitude, name) VALUES (?, ?, ?);
";

impl RoswaalLocationsStorage {
    pub async fn save(&self, locations: &Vec<RoswaalLocation>) -> Result<()> {
        let mut transaction = self.sqlite.transaction().await?;
        with_transaction!(transaction, async {
            let statements = locations.iter()
                .map(|_| UPSERT_QUERY)
                .collect::<Vec<&str>>()
                .join("\n");
            let mut bulk_insert_query = query::<Sqlite>(&statements);
            for location in locations.iter() {
                bulk_insert_query = bulk_insert_query.bind(location.coordinate().latitude())
                    .bind(location.coordinate().longitude())
                    .bind(&location.name().raw_value)
            }
            bulk_insert_query.execute(transaction.connection()).await?;
            Ok(())
        })
    }

    pub async fn all_in_alphabetical_order(&self) -> Result<Vec<RoswaalLocation>> {
        let mut transaction = self.sqlite.transaction().await?;
        with_transaction!(transaction, async {
            let locations = query_as::<Sqlite, SqliteLocation>(
                "SELECT * FROM Locations ORDER BY name"
            )
            .fetch_all(transaction.connection())
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
        })
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

    use crate::location::{coordinate::LocationCoordinate2D, location::RoswaalLocation, name::RoswaalLocationName};

    use super::*;

    #[tokio::test]
    async fn test_add_and_load_locations_no_prior_locations() {
        let storage = RoswaalLocationsStorage::new(RoswaalSqlite::in_memory().await.unwrap());
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
        _ = storage.save(&locations).await;
        let saved_locations = storage.all_in_alphabetical_order().await.unwrap();
        assert_eq!(locations, saved_locations)
    }

    #[tokio::test]
    async fn test_add_and_load_locations_upserts_prior_locations() {
        let storage = RoswaalLocationsStorage::new(RoswaalSqlite::in_memory().await.unwrap());
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
        _ = storage.save(&locations).await;
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
        _ = storage.save(&locations).await;
        let saved_locations = storage.all_in_alphabetical_order().await.unwrap();
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
