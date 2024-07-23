use std::borrow::Borrow;

use crate::{git::branch_name, location::storage::RoswaalStoredLocation, operations::load_all_locations::LoadAllLocationsStatus};

use super::ui_lib::{block_kit_views::{SlackHeader, SlackSection}, empty_view::EmptySlackView, slack_view::SlackView};

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
                EmptySlackView.flat_chain_blocks(
                    locations.iter()
                        .map(|location| LocationView { location })
                        .collect()
                )
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
        let mut body = format!("🏔️ *{}*\n", self.location.location().name().raw_name());
        body.push_str(
            &format!(
                "Latitude: _{:.8}_ Longitude: _{:.8}_\n",
                self.location.location().coordinate().latitude(),
                self.location.location().coordinate().longitude()
            )
        );
        if let Some(branch_name) = self.location.unmerged_branch_name() {
            body.push_str(&format!("_(Branch: {})_", branch_name.to_string()))
        }
        SlackSection::from_markdown(&body)
    }
}

#[cfg(test)]
mod tests {
    use crate::{git::branch_name::RoswaalOwnedGitBranchName, location::{location::RoswaalLocation, storage::RoswaalStoredLocation}, operations::load_all_locations::LoadAllLocationsStatus, slack::{test_support::SlackTestConstantBranches, ui_lib::test_support::assert_slack_view_snapshot}};

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
            false
        )
    }

    #[test]
    fn no_locations_snapshot() {
        assert_slack_view_snapshot(
            "locations-list-no-locations",
            &LocationsListView::new(LoadAllLocationsStatus::NoLocations),
            false
        )
    }
}
