use std::borrow::Borrow;

use crate::{location::storage::RoswaalStoredLocation, operations::load_all_locations::LoadAllLocationsStatus};

use super::{branch_name_view::OptionalBranchNameView, ui_lib::{block_kit_views::{SlackDivider, SlackHeader, SlackSection}, empty_view::EmptySlackView, for_each_view::ForEachView, if_let_view::IfLet, if_view::If, slack_view::SlackView}};

pub struct LocationsListView {
    status: LoadAllLocationsStatus
}

impl LocationsListView {
    pub fn new(status: LoadAllLocationsStatus) -> Self {
        Self { status }
    }
}

impl SlackView for LocationsListView {
    fn slack_body(&self) -> impl SlackView {
        SlackHeader::new("Locations")
            .flat_chain_block(self.status_view())
    }
}

impl LocationsListView {
    fn status_view(&self) -> impl SlackView {
        match self.status.borrow() {
            LoadAllLocationsStatus::Success(locations) => {
                ForEachView::new(locations.iter().enumerate(), |(index, location)| {
                    LocationView { location }
                        .flat_chain_block(If::is_true(*index < locations.len() - 1, || SlackDivider))
                })
                .erase_to_any_view()
            },
            LoadAllLocationsStatus::NoLocations => {
                SlackSection::from_markdown("No locations were fooooound!")
                    .erase_to_any_view()
            },
        }
    }
}

struct LocationView<'l> {
    location: &'l RoswaalStoredLocation
}

impl <'l> SlackView for LocationView<'l> {
    fn slack_body(&self) -> impl SlackView {
        SlackSection::from_markdown(
            &format!("🏔️ *{}*\n", self.location.location().name().raw_name())
        )
        .flat_chain_block(
            SlackSection::from_markdown(
                &format!(
                    "*Latitude:* {:.8}\n*Longitude:* {:.8}\n",
                    self.location.location().coordinate().latitude(),
                    self.location.location().coordinate().longitude()
                )
            )
        )
        .flat_chain_block(OptionalBranchNameView::new(self.location.unmerged_branch_name()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{location::{location::RoswaalLocation, storage::RoswaalStoredLocation}, operations::load_all_locations::LoadAllLocationsStatus, slack::{test_support::SlackTestConstantBranches, ui_lib::test_support::{assert_slack_view_snapshot, SnapshotMode}}};

    use super::LocationsListView;

    #[test]
    fn success_snapshot() {
        let branches = SlackTestConstantBranches::load();
        let locations = vec![
            RoswaalStoredLocation::new(
                RoswaalLocation::new_without_validation("Chetan's House", 50.0, 50.0),
                None
            ),
            RoswaalStoredLocation::new(
                RoswaalLocation::new_without_validation("McDonalds", 45.0, -50.0),
                Some(branches.add_locations().clone())
            )
        ];
        assert_slack_view_snapshot(
            "locations-list-success",
            &LocationsListView::new(LoadAllLocationsStatus::Success(locations)),
            SnapshotMode::Comparing
        )
    }

    #[test]
    fn no_locations_snapshot() {
        assert_slack_view_snapshot(
            "locations-list-no-locations",
            &LocationsListView::new(LoadAllLocationsStatus::NoLocations),
            SnapshotMode::Comparing
        )
    }
}
