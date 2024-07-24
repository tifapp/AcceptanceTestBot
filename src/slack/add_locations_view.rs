use std::borrow::Borrow;

use crate::{location::location::{RoswaalLocationStringError, RoswaalStringLocations}, operations::add_locations::AddLocationsStatus};

use super::{merge_conflict_view::MergeConflictView, pr_open_fail_view::FailedToOpenPullRequestView, ui_lib::{block_kit_views::{SlackDivider, SlackHeader, SlackSection}, empty_view::EmptySlackView, if_view::If, slack_view::SlackView}, users::MATTHEW_SLACK_USER_ID, warn_undeleted_branch_view::WarnUndeletedBranchView};

/// A view for adding locations.
pub struct AddLocationsView {
    status: AddLocationsStatus
}

impl AddLocationsView {
    pub fn new(status: AddLocationsStatus) -> Self {
        Self { status }
    }
}

impl SlackView for AddLocationsView {
    fn slack_body(&self) -> impl SlackView {
        SlackHeader::new("Add Locations")
            .flat_chain_block(self.status_view())
    }
}

impl AddLocationsView {
    fn status_view(&self) -> impl SlackView {
        match self.status.borrow() {
            AddLocationsStatus::Success { locations, did_delete_branch } => {
                If::is_true(
                    locations.has_valid_locations(),
                    || self.success_locations_view(locations)
                )
                .flat_chain_block(
                    If::is_true(locations.has_errors(), || self.failure_locations_view(locations))
                )
                .flat_chain_block(
                    If::is_true(
                        !did_delete_branch,
                        || SlackDivider.flat_chain_block(WarnUndeletedBranchView)
                    )
                )
                .erase_to_any_view()
            },
            AddLocationsStatus::NoLocationsAdded => {
                SlackSection::from_markdown("No Locations were aaaaaaaaaaadded.")
                    .erase_to_any_view()
            },
            AddLocationsStatus::FailedToOpenPullRequest => {
                FailedToOpenPullRequestView.erase_to_any_view()
            },
            AddLocationsStatus::MergeConflict => {
                MergeConflictView::new(MATTHEW_SLACK_USER_ID).erase_to_any_view()
            },
        }
    }

    fn success_locations_view(&self, locations: &RoswaalStringLocations) -> impl SlackView {
        let mut body = "✅ The following locations were added succeeeeeeesfully!\n".to_string();
        for location in locations.locations() {
            let line = format!(
                "- *{}* (Latitude: {:.8}, Longitude: {:.8})\n",
                location.name().raw_name(),
                location.coordinate().latitude(),
                location.coordinate().longitude()
            );
            body.push_str(&line)
        }
        SlackSection::from_markdown(&body)
    }

    fn failure_locations_view(&self, string_locations: &RoswaalStringLocations) -> impl SlackView {
        let mut body = "⚠️ The following locations were invaaaaaaalid...\n".to_string();
        for error in string_locations.errors() {
            body.push_str(&format!("- *{}* ", error.raw_associated_location_name()));
            match error {
                RoswaalLocationStringError::InvalidName(_, _) => {
                    body.push_str("(Invalid Name)")
                },
                RoswaalLocationStringError::InvalidCoordinate { name: _ } => {
                    body.push_str("(Invalid Coordinate)")
                }
            };
            body.push_str("\n")
        }
        SlackSection::from_markdown(&body)
    }
}

#[cfg(test)]
mod tests {
    use crate::{location::location::RoswaalStringLocations, operations::add_locations::AddLocationsStatus, slack::ui_lib::test_support::assert_slack_view_snapshot};

    use super::AddLocationsView;

    #[test]
    fn success_snapshot() {
        let string = "\
Antarctica, 50.20982098092, 50.09830883
New York
12.298739, 122.2989379
";
        let locations = RoswaalStringLocations::from_roswaal_locations_str(string);
        assert_slack_view_snapshot(
            "add-locations-success",
            &AddLocationsView::new(AddLocationsStatus::Success { locations, did_delete_branch: true }),
            false
        )
    }

    #[test]
    fn success_warn_undeleted_branch_snapshot() {
        let string = "\
Antarctica, 50.20982098092, 50.09830883
New York
12.298739, 122.2989379
";
        let locations = RoswaalStringLocations::from_roswaal_locations_str(string);
        assert_slack_view_snapshot(
            "add-locations-warn-undeleted-branch",
            &AddLocationsView::new(AddLocationsStatus::Success { locations, did_delete_branch: false }),
            false
        )
    }

    #[test]
    fn success_with_no_failures_snapshot() {
        let string = "\
Antarctica, 50.20982098092, 50.09830883
";
        let locations = RoswaalStringLocations::from_roswaal_locations_str(string);
        assert_slack_view_snapshot(
            "add-locations-no-failures",
            &AddLocationsView::new(AddLocationsStatus::Success { locations, did_delete_branch: true }),
            false
        )
    }

    #[test]
    fn success_with_no_successes_snapshot() {
        let string = "\
50.20982098092, 50.09830883
";
        let locations = RoswaalStringLocations::from_roswaal_locations_str(string);
        assert_slack_view_snapshot(
            "add-locations-no-successes",
            &AddLocationsView::new(AddLocationsStatus::Success { locations, did_delete_branch: true }),
            false
        )
    }

    #[test]
    fn no_locations_snapshot() {
        assert_slack_view_snapshot(
            "add-locations-no-locations",
            &AddLocationsView::new(AddLocationsStatus::NoLocationsAdded),
            false
        )
    }

    #[test]
    fn pr_fail_snapshot() {
        assert_slack_view_snapshot(
            "add-locations-pr-fail",
            &AddLocationsView::new(AddLocationsStatus::FailedToOpenPullRequest),
            false
        )
    }

    #[test]
    fn merge_conflict_snapshot() {
        assert_slack_view_snapshot(
            "add-locations-merge-conflict",
            &AddLocationsView::new(AddLocationsStatus::MergeConflict),
            false
        )
    }
}
