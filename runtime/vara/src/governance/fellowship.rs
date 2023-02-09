// This file is part of Gear.

// Copyright (C) 2021-2022 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Elements of governance concerning the Gear Fellowship - an on-chain organ
//! that acts as an oracle to certify whether a proposal is good and can be
//! submitted within an agile track yet being of high impact

use frame_support::traits::MapSuccess;
use sp_runtime::{
    morph_types,
    traits::{CheckedSub, ConstU16, Replace, TypedGet},
};

use super::*;
use crate::{DAYS, DOLLARS};

parameter_types! {
    pub const AlarmInterval: BlockNumber = 1;
    pub const SubmissionDeposit: Balance = 0;
    pub const UndecidingTimeout: BlockNumber = 7 * DAYS;
}

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
    type Id = u16;
    type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
    fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
        static DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 4] = [
            (
                0u16,
                pallet_referenda::TrackInfo {
                    name: "candidates",
                    max_deciding: 10,
                    decision_deposit: 100 * DOLLARS,
                    prepare_period: 30 * MINUTES,
                    decision_period: 7 * DAYS,
                    confirm_period: 30 * MINUTES,
                    min_enactment_period: MINUTES,
                    min_approval: pallet_referenda::Curve::LinearDecreasing {
                        length: Perbill::from_percent(100),
                        floor: Perbill::from_percent(50),
                        ceil: Perbill::from_percent(100),
                    },
                    min_support: pallet_referenda::Curve::LinearDecreasing {
                        length: Perbill::from_percent(100),
                        floor: Perbill::from_percent(0),
                        ceil: Perbill::from_percent(50),
                    },
                },
            ),
            (
                1u16,
                pallet_referenda::TrackInfo {
                    name: "fellows",
                    max_deciding: 10,
                    decision_deposit: 10 * DOLLARS,
                    prepare_period: 30 * MINUTES,
                    decision_period: 7 * DAYS,
                    confirm_period: 30 * MINUTES,
                    min_enactment_period: MINUTES,
                    min_approval: pallet_referenda::Curve::LinearDecreasing {
                        length: Perbill::from_percent(100),
                        floor: Perbill::from_percent(50),
                        ceil: Perbill::from_percent(100),
                    },
                    min_support: pallet_referenda::Curve::LinearDecreasing {
                        length: Perbill::from_percent(100),
                        floor: Perbill::from_percent(0),
                        ceil: Perbill::from_percent(50),
                    },
                },
            ),
            (
                2u16,
                pallet_referenda::TrackInfo {
                    name: "experts",
                    max_deciding: 10,
                    decision_deposit: DOLLARS,
                    prepare_period: 30 * MINUTES,
                    decision_period: 7 * DAYS,
                    confirm_period: 30 * MINUTES,
                    min_enactment_period: MINUTES,
                    min_approval: pallet_referenda::Curve::LinearDecreasing {
                        length: Perbill::from_percent(100),
                        floor: Perbill::from_percent(50),
                        ceil: Perbill::from_percent(100),
                    },
                    min_support: pallet_referenda::Curve::LinearDecreasing {
                        length: Perbill::from_percent(100),
                        floor: Perbill::from_percent(0),
                        ceil: Perbill::from_percent(50),
                    },
                },
            ),
            (
                3u16,
                pallet_referenda::TrackInfo {
                    name: "masters",
                    max_deciding: 10,
                    decision_deposit: DOLLARS,
                    prepare_period: 30 * MINUTES,
                    decision_period: 7 * DAYS,
                    confirm_period: 30 * MINUTES,
                    min_enactment_period: MINUTES,
                    min_approval: pallet_referenda::Curve::LinearDecreasing {
                        length: Perbill::from_percent(100),
                        floor: Perbill::from_percent(50),
                        ceil: Perbill::from_percent(100),
                    },
                    min_support: pallet_referenda::Curve::LinearDecreasing {
                        length: Perbill::from_percent(100),
                        floor: Perbill::from_percent(0),
                        ceil: Perbill::from_percent(50),
                    },
                },
            ),
        ];
        &DATA[..]
    }

    fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
        use super::origins::Origin;

        #[cfg(feature = "runtime-benchmarks")]
        {
            // For benchmarks, we enable a root origin.
            // It is important that this is not available in production!
            let root: Self::RuntimeOrigin = frame_system::RawOrigin::Root.into();
            if &root == id {
                return Ok(9);
            }
        }

        match Origin::try_from(id.clone()) {
            Ok(Origin::FellowshipInitiates) => Ok(0),
            Ok(Origin::Fellows) => Ok(1),
            Ok(Origin::FellowshipExperts) => Ok(2),
            Ok(Origin::FellowshipMasters) => Ok(3),
            _ => Err(()),
        }
    }
}
pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);

pub type FellowshipReferendaInstance = pallet_referenda::Instance2;

impl pallet_referenda::Config<FellowshipReferendaInstance> for Runtime {
    type WeightInfo = pallet_referenda::weights::SubstrateWeight<Self>;
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type Scheduler = Scheduler;
    type Currency = Balances;
    type SubmitOrigin =
        pallet_ranked_collective::EnsureMember<Runtime, FellowshipCollectiveInstance, 1>;
    type CancelOrigin = FellowshipExperts;
    type KillOrigin = FellowshipMasters;
    type Slash = ();
    type Votes = pallet_ranked_collective::Votes;
    type Tally = pallet_ranked_collective::TallyOf<Runtime, FellowshipCollectiveInstance>;
    type SubmissionDeposit = SubmissionDeposit;
    type MaxQueued = ConstU32<100>;
    type UndecidingTimeout = UndecidingTimeout;
    type AlarmInterval = AlarmInterval;
    type Tracks = TracksInfo;
    type Preimages = Preimage;
}

pub type FellowshipCollectiveInstance = pallet_ranked_collective::Instance1;

morph_types! {
    /// A `TryMorph` implementation to reduce a scalar by a particular amount, checking for
    /// underflow.
    pub type CheckedReduceBy<N: TypedGet>: TryMorph = |r: N::Type| -> Result<N::Type, ()> {
        r.checked_sub(&N::get()).ok_or(())
    } where N::Type: CheckedSub;
}

impl pallet_ranked_collective::Config<FellowshipCollectiveInstance> for Runtime {
    type WeightInfo = pallet_ranked_collective::weights::SubstrateWeight<Self>;
    type RuntimeEvent = RuntimeEvent;
    // Promotion is by any of:
    // - Root can demote arbitrarily.
    // - the FellowshipAdmin origin (i.e. token holder referendum);
    type PromoteOrigin = EitherOf<
        frame_system::EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>,
        MapSuccess<FellowshipAdmin, Replace<ConstU16<3>>>,
    >;
    // Demotion is by any of:
    // - Root can demote arbitrarily.
    // - the FellowshipAdmin origin (i.e. token holder referendum);
    type DemoteOrigin = EitherOf<
        frame_system::EnsureRootWithSuccess<Self::AccountId, ConstU16<65535>>,
        MapSuccess<FellowshipAdmin, Replace<ConstU16<3>>>,
    >;
    type Polls = FellowshipReferenda;
    type MinRankOfClass = sp_runtime::traits::Identity;
    type VoteWeight = pallet_ranked_collective::Geometric;
}
