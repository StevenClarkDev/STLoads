@extends('layout.app')
@section('content')
    <div>
        <div class="page-title">
            <div class="row">
                <div class="col-6">
                    <h4>Manage Loads</h4>
                </div>
                <div class="col-6">
                    <ol class="breadcrumb">
                        <li class="breadcrumb-item"><a href="dashboard">
                                <svg class="stroke-icon">
                                    <use href="{{ url('/assets/svg/icon-sprite.svg#stroke-home') }}"></use>
                                </svg></a></li>
                        <li class="breadcrumb-item active">Manage Loads</li>
                    </ol>
                </div>
            </div>
        </div>
    </div>
    <div>
        <div class="row">
            <div class="col-sm-12">
                <div class="card">
                    <div class="card-body p-0">
                        <div class="mx-3">
                            <div class="card-header pb-0 card-no-border">
                                <div class="d-flex justify-content-between align-items-center flex-wrap">
                                    <div class="mb-2">
                                        <h4 class="mb-1 text-start">Loads List</h4>
                                        <span>See Registered Loads below.</span>
                                    </div>
                                    <div class="d-flex gap-2">
                                        <button id="resetPrefsBtn" class="btn btn-sm btn-secondary px-2 d-none"
                                            type="button" data-bs-toggle="modal" data-bs-target="#recommendationModal"
                                            title="Load Preferences">
                                            Load Preferences
                                        </button>
                                        @if ($roleId == 2 || $roleId == 4 || $roleId == 5)
                                            <a href="{{ route('loads.add') }}" class="btn btn-sm btn-primary px-3"
                                                type="button">
                                                <i class="bi bi-plus-circle me-1"></i> Add Load
                                            </a>
                                        @endif
                                        {{-- <button class="btn btn-sm btn-outline-primary px-3" type="button"
                                            data-bs-toggle="collapse" data-bs-target="#collapseProduct"
                                            aria-expanded="false" aria-controls="collapseProduct">
                                            <i class="bi bi-filter me-1"></i> Filter
                                        </button>
                                        <button class="btn btn-sm btn-outline-primary px-3" onclick="exportToExcel()">
                                            <i class="bi bi-download me-1"></i> Export
                                        </button> --}}
                                    </div>
                                </div>
                            </div>
                            <div class="collapse" id="collapseProduct">
                                <div class="card card-body list-product-body">
                                    <form id="filterForm">
                                        <div class="row align-items-end g-2">
                                            <div class="col-md-3">
                                                <label class="form-label small">Status</label>
                                                <select class="form-select form-select-sm" name="status">
                                                    <option value="">All</option>
                                                    <option value="pending">Pending</option>
                                                    <option value="accepted">Accepted</option>
                                                    <option value="rejected">Rejected</option>
                                                </select>
                                            </div>
                                            <div class="col-md-3">
                                                <label class="form-label small">Payment</label>
                                                <select class="form-select form-select-sm" name="payment">
                                                    <option value="">All</option>
                                                    <option value="paid">Paid</option>
                                                    <option value="unpaid">Unpaid</option>
                                                    <option value="pending">Pending</option>
                                                </select>
                                            </div>
                                            <div class="col-md-3">
                                                <label class="form-label small">From</label>
                                                <input type="text" class="form-control form-control-sm datetimepicker"
                                                    name="from">
                                            </div>
                                            <div class="col-md-3">
                                                <label class="form-label small">To</label>
                                                <input type="text" class="form-control form-control-sm datetimepicker"
                                                    name="to">
                                            </div>
                                            <div class="col-md-12 text-end">
                                                <button type="submit" class="btn btn-sm btn-primary px-4">Apply
                                                    Filters</button>
                                            </div>
                                        </div>
                                    </form>
                                </div>
                            </div>

                            <div class="card-body">
                                <div class="list-product-header">
                                    <div>
                                        <button class="btn btn-sm btn-outline-light rounded-4 border active"
                                            onclick="switchTab(this, 'all')">All Loads ({{ $loadCount }})</button>
                                        @if ($roleId == 3)
                                            <button class="btn btn-sm btn-outline-light rounded-4 border"
                                                onclick="switchTab(this, 'recommended')">Recommended Loads
                                                ({{ $recommendedLoadLegsCount }})</button>
                                            {{-- <button class="btn btn-sm btn-outline-light rounded-4 border"
                                                onclick="switchTab(this, 'accepted')">Accepted Loads (211)</button>
                                            <button class="btn btn-sm btn-outline-light rounded-4 border"
                                                onclick="switchTab(this, 'time')">Time-Sensitive (48)</button> --}}
                                        @endif
                                    </div>
                                </div>
                            </div>
                            <div class="card-body tab-content">
                                <div class="tab-pane fade show active" id="tab-all">
                                    <div class="table-responsive">
                                        <table class="table table-striped align-middle text-nowrap" id="user-approval-table"
                                            style="font-size: 0.875rem;">
                                            <thead class="bg-white" style="position: sticky; top: 0; z-index: 2;">
                                                <tr>
                                                    <th>Load ID</th>
                                                    <th>Origin</th>
                                                    <th>Destination</th>
                                                    <th>Pickup Date</th>
                                                    <th>Delivery Date</th>
                                                    <th>Status</th>
                                                    @if ($roleId != 3)
                                                        <th>Remarks</th>
                                                        <th>Carrier</th>
                                                    @endif
                                                    <th>Bid Status</th>
                                                    <th>Amount</th>
                                                    <th>Payment</th>
                                                    <th>Action</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                @if(count($load_legs) > 0)
                                                    @foreach ($load_legs as $i => $load_leg)
                                                        <tr>
                                                            <td>{{ $load_leg->leg_code }}</td>
                                                            <td>
                                                                @php
                                                                    $pickupTitle = $load_leg->pickupLocation?->address ?? $load_leg->pickupLocation?->name;
                                                                    if ($load_leg->pickupLocation?->city && $load_leg->pickupLocation?->country) {
                                                                        $pickupTitle = $load_leg->pickupLocation->name . ' - ' . $load_leg->pickupLocation->city->name . ' - ' . $load_leg->pickupLocation->country->name;
                                                                    }
                                                                @endphp
                                                                <span class="badge rounded-circle p-2 badge-primary"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $pickupTitle }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                            </td>
                                                            <td>
                                                                @php
                                                                    $deliveryTitle = $load_leg->deliveryLocation?->address ?? $load_leg->deliveryLocation?->name;
                                                                    if ($load_leg->deliveryLocation?->city && $load_leg->deliveryLocation?->country) {
                                                                        $deliveryTitle = $load_leg->deliveryLocation->name . ' - ' . $load_leg->deliveryLocation->city->name . ' - ' . $load_leg->deliveryLocation->country->name;
                                                                    }
                                                                @endphp
                                                                <span class="badge rounded-circle p-2 badge-primary"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $deliveryTitle }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                            </td>
                                                            <td>{{ \Carbon\Carbon::parse($load_leg->pickup_date)->format('jS M, Y') }}
                                                            </td>
                                                            <td>{{ \Carbon\Carbon::parse($load_leg->delivery_date)->format('jS M, Y') }}
                                                            </td>
                                                            <td>
                                                                <span
                                                                    class="badge rounded-pill bg-warning p-2 text-capitalize">{{ $load_leg->status_master?->name }}</span>
                                                            </td>
                                                            @if ($roleId != 3)
                                                                <td>
                                                                    @if ($load_leg->status_id == 0 || $load_leg->status_id == 7)
                                                                        {{ $load_leg->load_master->latestHistory?->remarks ?? 'No remarks provided.' }}
                                                                    @else
                                                                        No remarks provided.
                                                                    @endif
                                                                </td>
                                                                <td>
                                                                    @if ($load_leg->carrier)
                                                                        {{ $load_leg->carrier->name }}
                                                                    @else
                                                                        -
                                                                    @endif
                                                                </td>
                                                            @endif
                                                            <td>
                                                                @if ($load_leg->bid_status == 'Fixed')
                                                                    <span
                                                                        class="badge rounded-pill bg-primary p-2 text-capitalize">{{ $load_leg->bid_status }}</span>
                                                                @else
                                                                    <span
                                                                        class="badge rounded-pill bg-info p-2 text-capitalize">{{ $load_leg->bid_status }}</span>
                                                                @endif
                                                            </td>
                                                            <td>
                                                                @if ($load_leg->status_id >= 4 || $roleId != 3)
                                                                    <button class="btn btn-primary btn-sm fix-width">
                                                                        ${{ number_format($load_leg->booked_amount > 0 ? $load_leg->booked_amount : $load_leg->price, 0) }}
                                                                    </button>
                                                                @elseif ($load_leg->bid_status == 'Fixed')
                                                                    <button class="btn btn-primary btn-sm fix-width"
                                                                        data-bs-toggle="modal" data-bs-target="#confirmFixedModal"
                                                                        data-book-url="{{ route('load-legs.book', $load_leg) }}"
                                                                        data-amount="{{ $load_leg->price }}"
                                                                        data-leg-code="{{ $load_leg->leg_code ?? '' }}">
                                                                        ${{ number_format($load_leg->price, 0) }}
                                                                    </button>
                                                                @else
                                                                    <button class="btn btn-outline-primary btn-sm fix-width"
                                                                        data-bs-toggle="modal" data-bs-target="#bidModal-{{ $i }}">
                                                                        ${{ number_format($load_leg->booked_amount > 0 ? $load_leg->booked_amount : $load_leg->price, 0) }}
                                                                    </button>
                                                                @endif

                                                                {{-- Reusable Fixed-Price Booking Modal --}}
                                                                <div class="modal fade" id="confirmFixedModal" tabindex="-1"
                                                                    aria-hidden="true">
                                                                    <div class="modal-dialog">
                                                                        <form id="confirmFixedForm" class="modal-content"
                                                                            method="POST" action="#">
                                                                            @csrf
                                                                            <div class="modal-header">
                                                                                <h5 class="modal-title">Book this load?</h5>
                                                                                <button type="button" class="btn-close"
                                                                                    data-bs-dismiss="modal"></button>
                                                                            </div>

                                                                            <div class="modal-body">
                                                                                <p class="mb-2">
                                                                                    You’re about to <strong>book</strong>
                                                                                    <span id="fixedLegLabel"></span>
                                                                                    at <strong id="fixedAmountLabel"></strong>.
                                                                                </p>
                                                                                <p class="text-muted small mb-0">
                                                                                    This will reserve the load at the fixed
                                                                                    price.
                                                                                </p>

                                                                                {{-- Hidden value if backend expects it --}}
                                                                                <input type="hidden" name="amount"
                                                                                    id="fixedAmountInput" value="">
                                                                            </div>

                                                                            <div class="modal-footer">
                                                                                <button class="btn btn-light" type="button"
                                                                                    data-bs-dismiss="modal">Cancel</button>
                                                                                <button class="btn btn-primary" id="confirmFixedBtn"
                                                                                    type="submit">
                                                                                    Proceed
                                                                                </button>
                                                                            </div>
                                                                        </form>
                                                                    </div>
                                                                </div>


                                                                <!-- Bid Modal -->
                                                                <div class="modal fade" id="bidModal-{{ $i }}" tabindex="-1"
                                                                    aria-hidden="true">
                                                                    <div class="modal-dialog modal-dialog-centered"
                                                                        style="max-width: 600px;">
                                                                        <div class="modal-content p-4">
                                                                            <div class="modal-header border-0">
                                                                                <h5 class="modal-title">Submit Your Bid</h5>
                                                                                <button type="button" class="btn-close"
                                                                                    data-bs-dismiss="modal"
                                                                                    aria-label="Close"></button>
                                                                            </div>

                                                                            <form method="POST"
                                                                                action="{{ route('loads.bid', $load_leg->id) }}">
                                                                                @csrf
                                                                                <div class="modal-body">
                                                                                    <p class="text-muted mb-4">Please review
                                                                                        the client's offer and submit your bid
                                                                                        below.</p>
                                                                                    <div class="row my-3">
                                                                                        <div class="col-md-6">
                                                                                            <label class="form-label">Client
                                                                                                Price</label>
                                                                                            <input class="form-control"
                                                                                                value="${{ number_format($load_leg->price, 0) }}"
                                                                                                readonly>
                                                                                        </div>
                                                                                        <div class="col-md-6">
                                                                                            <label class="form-label">Your
                                                                                                Bid</label>
                                                                                            <input type="number" min="1" step="1"
                                                                                                name="amount" class="form-control"
                                                                                                placeholder="Enter your bid"
                                                                                                required>
                                                                                        </div>
                                                                                    </div>
                                                                                    <label class="form-label mt-2">Note
                                                                                        (optional)
                                                                                    </label>
                                                                                    <input type="text" name="note"
                                                                                        class="form-control"
                                                                                        placeholder="Any additional info">
                                                                                </div>
                                                                                @if ($roleId == 3)
                                                                                    <div
                                                                                        class="modal-footer border-0 d-flex justify-content-end gap-2">
                                                                                        <button type="button"
                                                                                            class="btn btn-outline-secondary"
                                                                                            data-bs-dismiss="modal">Cancel</button>
                                                                                        <button type="submit"
                                                                                            class="btn btn-primary">Submit Bid
                                                                                            &amp; Chat</button>
                                                                                    </div>
                                                                                @endif
                                                                            </form>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </td>
                                                            @php
                                                                $esc = $load_leg->escrow ?? null;
                                                                $isOwner = $user->id == $load_leg->load_master->user_id;
                                                                $displayAmount = $load_leg->booked_amount > 0 ? $load_leg->booked_amount : $load_leg->price;
                                                            @endphp
                                                            <td>
                                                                @if ($isOwner && (int)$load_leg->status_id === 4)
                                                                    {{-- Booked but not initiated --}}
                                                                    <button type="button" class="btn btn-sm btn-success open-pay-modal"
                                                                            data-leg-id="{{ $load_leg->id }}"
                                                                            data-leg-code="{{ $load_leg->leg_code }}"
                                                                            data-amount="{{ $displayAmount }}"
                                                                            data-fund-url="{{ route('legs.escrow.fund', $load_leg->id) }}">
                                                                        Pay
                                                                    </button>

                                                                @elseif ((int)$load_leg->status_id >= 5)
                                                                    @if ($esc && $esc->status === 'funded')
                                                                        <span class="badge rounded-pill badge-light-success p-2">Paid</span>
                                                                    @elseif ($esc && $esc->status === 'released')
                                                                        <span class="badge rounded-pill badge-light-success p-2">Released</span>
                                                                    @elseif($isOwner)
                                                                        {{-- uninitiated/unfunded/failed/null → allow retry --}}
                                                                        <button type="button" class="btn btn-sm btn-success open-pay-modal"
                                                                                data-leg-id="{{ $load_leg->id }}"
                                                                                data-leg-code="{{ $load_leg->leg_code }}"
                                                                                data-amount="{{ $displayAmount }}"
                                                                                data-fund-url="{{ route('legs.escrow.fund', $load_leg->id) }}">
                                                                            {{ $esc && $esc->status === 'failed' ? 'Failed — Try again' : 'Unpaid — Pay now' }}
                                                                        </button>
                                                                    @else
                                                                        <span class="badge rounded-pill badge-light-warning p-2">Pending</span>
                                                                    @endif
                                                                @else
                                                                    <span class="badge rounded-pill badge-light-warning p-2">Pending</span>
                                                                @endif
                                                            </td>
                                                            <td>
                                                                @if ($roleId != 3)
                                                                    <a href="{{ route('loads.view', $load_leg->load_master->id) }}"
                                                                        class="btn align-items-center">
                                                                        <i class="bi bi-eye text-primary"></i>
                                                                    </a>
                                                                @elseif($roleId == 3 && $load_leg->status_id == 8 && $load_leg->booked_carrier_id == $user->id && !$load_leg->pickup_started_at)
                                                                    <form method="POST"
                                                                        action="{{ route('leg.pickup.start', $load_leg->id) }}">
                                                                        @csrf
                                                                        <button class="btn btn-success btn-sm"
                                                                            type="submit">
                                                                            Start Pickup
                                                                        </button>
                                                                    </form>
                                                                @endif
                                                                @if ($load_leg->status_id >= 5)
                                                                    <a href="{{ route('leg.track', $load_leg->id) }}" class="btn btn-info btn-sm">
                                                                        Track Leg
                                                                    </a>
                                                                @endif
                                                            </td>
                                                        </tr>
                                                    @endforeach
                                                @else
                                                    <tr>
                                                        <td colspan="{{ $roleId != 3 ? 11 : 9 }}" class="text-center py-4">
                                                            No loads found.
                                                        </td>
                                                    </tr>
                                                @endif
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                                {{-- Stripe Pay Modal --}}
                                <div class="modal fade" id="payModal" tabindex="-1" aria-hidden="true">
                                    <div class="modal-dialog modal-dialog-centered" style="max-width: 520px;">
                                        <div class="modal-content p-3">
                                            <div class="modal-header">
                                                <h5 class="modal-title">
                                                    Pay for Load <span id="payModalLegLabel" class="text-muted"></span>
                                                </h5>
                                                <button type="button" class="btn-close" data-bs-dismiss="modal"
                                                    aria-label="Close"></button>
                                            </div>

                                            <div class="modal-body">
                                                <div class="mb-2 small text-muted">
                                                    Amount: <strong id="payModalAmountLabel">—</strong>
                                                </div>

                                                <div id="card-element-container" class="border rounded p-3">
                                                    <div id="card-element"></div>
                                                </div>

                                                <div id="payError" class="alert alert-danger d-none mt-3"></div>
                                                <div id="paySuccess" class="alert alert-success d-none mt-3"></div>
                                            </div>

                                            <div class="modal-footer">
                                                <button type="button" class="btn btn-light"
                                                    data-bs-dismiss="modal">Cancel</button>
                                                <button id="payConfirmBtn" type="button" class="btn btn-primary">
                                                    Pay now
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div class="tab-pane fade" id="tab-recommended">
                                    <div class="table-responsive">
                                        <table class="table table-striped align-middle text-nowrap"
                                            id="user-recommended-table" style="font-size: 0.875rem;">
                                            <thead class="bg-white" style="position: sticky; top: 0; z-index: 2;">
                                                <tr>
                                                    <th>Load ID</th>
                                                    <th>Pickup Location</th>
                                                    <th>Delivery Location</th>
                                                    <th>Equipment</th>
                                                    <th>Load Type</th>
                                                    <th>Weight</th>
                                                    <th>Pickup Date</th>
                                                    <th>Delivery Date</th>
                                                    <!-- <th>Score</th> -->
                                                    <th>Match Info</th>
                                                    <th>Status</th>
                                                    <th>Bid Status</th>
                                                    <th>Amount</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                @if($recommendedLoadLegs != null && count($recommendedLoadLegs) > 0)
                                                    @foreach ($recommendedLoadLegs as $i => $load_leg)
                                                        <tr>
                                                            <td>{{ $load_leg->leg_code }}</td>
                                                            <td>
                                                                @php
                                                                    $pickupTitle = $load_leg->pickupLocation?->address ?? $load_leg->pickupLocation?->name;
                                                                    if ($load_leg->pickupLocation?->city && $load_leg->pickupLocation?->country) {
                                                                        $pickupTitle = $load_leg->pickupLocation->name . ' - ' . $load_leg->pickupLocation->city->name . ' - ' . $load_leg->pickupLocation->country->name;
                                                                    }
                                                                @endphp
                                                                <span class="badge rounded-circle p-2 badge-primary"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $pickupTitle }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                            </td>
                                                            <td>
                                                                @php
                                                                    $deliveryTitle = $load_leg->deliveryLocation?->address ?? $load_leg->deliveryLocation?->name;
                                                                    if ($load_leg->deliveryLocation?->city && $load_leg->deliveryLocation?->country) {
                                                                        $deliveryTitle = $load_leg->deliveryLocation->name . ' - ' . $load_leg->deliveryLocation->city->name . ' - ' . $load_leg->deliveryLocation->country->name;
                                                                    }
                                                                @endphp
                                                                <span class="badge rounded-circle p-2 badge-primary"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $deliveryTitle }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                            </td>
                                                            <td>{{ $load_leg->load_master?->equipment?->name }}</td>
                                                            <td>{{ $load_leg->load_master?->load_type?->name }}</td>
                                                            <td>{{ $load_leg->load_master?->weight }}</td>
                                                            <td>{{ \Carbon\Carbon::parse($load_leg->pickup_date)->format('jS M, Y') }}
                                                            </td>
                                                            <td>{{ \Carbon\Carbon::parse($load_leg->delivery_date)->format('jS M, Y') }}
                                                            </td>
                                                            <td>
                                                                <button
                                                                    class="btn btn-sm show-ai-debug btn-outline-primary px-3 rounded-pill shadow-sm"
                                                                    data-leg="{{ $load_leg->leg_code }}"
                                                                    data-debug='@json($load_leg->debug_info)' data-bs-toggle="modal"
                                                                    data-bs-target="#aiDebugModal">
                                                                    {{ $load_leg->score }}
                                                                    <i class="bi bi-eye"></i>
                                                                </button>
                                                            </td>
                                                            <td>
                                                                <span
                                                                    class="badge rounded-pill bg-light-info p-2 text-capitalize text-primary">{{ $load_leg->status_master?->name }}</span>
                                                            </td>
                                                            <td>
                                                                @if ($load_leg->bid_status == 'Fixed')
                                                                    <span
                                                                        class="badge rounded-pill bg-primary p-2 text-capitalize">{{ $load_leg->bid_status }}</span>
                                                                @else
                                                                    <span
                                                                        class="badge rounded-pill bg-info p-2 text-capitalize">{{ $load_leg->bid_status }}</span>
                                                                @endif
                                                            </td>
                                                            <td>
                                                                @if ($load_leg->status_id == 4 || $roleId != 3)
                                                                    <button class="btn btn-primary btn-sm fix-width">
                                                                        ${{ number_format($load_leg->booked_amount > 0 ? $load_leg->booked_amount : $load_leg->price, 0) }}
                                                                    </button>
                                                                @elseif ($load_leg->bid_status == 'Fixed')
                                                                    <button class="btn btn-primary btn-sm fix-width"
                                                                        data-bs-toggle="modal" data-bs-target="#reCconfirmFixedModal"
                                                                        data-book-url="{{ route('load-legs.book', $load_leg) }}"
                                                                        data-amount="{{ $load_leg->price }}"
                                                                        data-leg-code="{{ $load_leg->leg_code ?? '' }}">
                                                                        ${{ number_format($load_leg->price, 0) }}
                                                                    </button>
                                                                @else
                                                                    <button class="btn btn-outline-primary btn-sm fix-width"
                                                                        data-bs-toggle="modal" data-bs-target="#bidrecModal-{{ $i }}">
                                                                        ${{ number_format($load_leg->price, 0) }}
                                                                    </button>
                                                                @endif

                                                                <div class="modal fade" id="reCconfirmFixedModal" tabindex="-1"
                                                                    aria-hidden="true">
                                                                    <div class="modal-dialog">
                                                                        <form id="reCconfirmFixedForm" class="modal-content"
                                                                            method="POST" action="#">
                                                                            @csrf
                                                                            <div class="modal-header">
                                                                                <h5 class="modal-title">Book this load?
                                                                                </h5>
                                                                                <button type="button" class="btn-close"
                                                                                    data-bs-dismiss="modal"></button>
                                                                            </div>

                                                                            <div class="modal-body">
                                                                                <p class="mb-2">
                                                                                    You’re about to <strong>book</strong>
                                                                                    <span id="reCfixedLegLabel"></span>
                                                                                    at <strong id="reCfixedAmountLabel"></strong>.
                                                                                </p>
                                                                                <p class="text-muted small mb-0">
                                                                                    This will reserve the load at the fixed
                                                                                    price.
                                                                                </p>

                                                                                {{-- Hidden value if backend expects it --}}
                                                                                <input type="hidden" name="amount"
                                                                                    id="reCfixedAmountInput" value="">
                                                                            </div>

                                                                            <div class="modal-footer">
                                                                                <button class="btn btn-light" type="button"
                                                                                    data-bs-dismiss="modal">Cancel</button>
                                                                                <button class="btn btn-primary"
                                                                                    id="reCconfirmFixedBtn" type="submit">
                                                                                    Proceed
                                                                                </button>
                                                                            </div>
                                                                        </form>
                                                                    </div>
                                                                </div>

                                                                <!-- Bid Modal -->
                                                                <div class="modal fade" id="bidrecModal-{{ $i }}" tabindex="-1"
                                                                    aria-hidden="true">
                                                                    <div class="modal-dialog modal-dialog-centered"
                                                                        style="max-width: 600px;">
                                                                        <div class="modal-content p-4">
                                                                            <div class="modal-header border-0">
                                                                                <h5 class="modal-title">Submit Your Bid
                                                                                </h5>
                                                                                <button type="button" class="btn-close"
                                                                                    data-bs-dismiss="modal"
                                                                                    aria-label="Close"></button>
                                                                            </div>

                                                                            <form method="POST"
                                                                                action="{{ route('loads.bid', $load_leg->id) }}">
                                                                                @csrf
                                                                                <div class="modal-body">
                                                                                    <p class="text-muted mb-4">Please
                                                                                        review
                                                                                        the client's offer and submit your
                                                                                        bid
                                                                                        below.</p>
                                                                                    <div class="row my-3">
                                                                                        <div class="col-md-6">
                                                                                            <label class="form-label">Client
                                                                                                Price</label>
                                                                                            <input class="form-control"
                                                                                                value="${{ number_format($load_leg->price, 0) }}"
                                                                                                readonly>
                                                                                        </div>
                                                                                        <div class="col-md-6">
                                                                                            <label class="form-label">Your
                                                                                                Bid</label>
                                                                                            <input type="number" min="1" step="1"
                                                                                                name="amount" class="form-control"
                                                                                                placeholder="Enter your bid"
                                                                                                required>
                                                                                        </div>
                                                                                    </div>
                                                                                    <label class="form-label mt-2">Note
                                                                                        (optional)
                                                                                    </label>
                                                                                    <input type="text" name="note"
                                                                                        class="form-control"
                                                                                        placeholder="Any additional info">
                                                                                </div>
                                                                                @if ($roleId == 3)
                                                                                    <div
                                                                                        class="modal-footer border-0 d-flex justify-content-end gap-2">
                                                                                        <button type="button"
                                                                                            class="btn btn-outline-secondary"
                                                                                            data-bs-dismiss="modal">Cancel</button>
                                                                                        <button type="submit"
                                                                                            class="btn btn-primary">Submit
                                                                                            Bid
                                                                                            &amp; Chat</button>
                                                                                    </div>
                                                                                @endif
                                                                            </form>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    @endforeach
                                                @elseif (!$user->carrierPreference)
                                                    <tr>
                                                        <td colspan="14"> Please Fill the Preference Form First </td>
                                                    </tr>
                                                @else
                                                    <tr>
                                                        <td colspan="14" class="text-center py-4">No recommended loads found.
                                                        </td>
                                                    </tr>
                                                @endif

                                            </tbody>
                                        </table>
                                    </div>
                                </div>

                                <!-- Accepted Loads Tab -->
                                {{-- <div class="tab-pane fade" id="tab-accepted">
                                    <h1>tab-accepted</h1>
                                </div> --}}

                                <!-- Time-Sensitive Tab -->
                                {{-- <div class="tab-pane fade" id="tab-time">
                                    <h1>tab-time</h1>
                                </div> --}}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <!-- Recommendation Preferences Modal -->
    <div class="modal fade" id="recommendationModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 800px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title">Your Load Preferences</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="recommendationForm">
                        <div class="row">
                            <div class="col-md-6">
                                <label class="form-label" for="equipment_owned">Equipment Owned</label>
                                <select class="form-select select2" id="equipment_owned" name="equipment_id[]" multiple>
                                    @foreach ($equipments as $equipment)
                                        <option value="{{ $equipment->id }}" @if (in_array($equipment->id, old('equipment_id', $carrierPreference->equipment_id ?? []))) selected @endif>
                                            {{ $equipment->name }}
                                        </option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="max_weight_capacity">Max Weight Capacity</label>
                                <input class="form-control" id="max_weight_capacity" type="number"
                                    name="max_weight_capacity"
                                    value="{{ old('max_weight_capacity', $carrierPreference->max_weight_capacity ?? '') }}">
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="load_type">Load Type</label>
                                <select class="form-select select2" id="load_type" name="load_type_id[]" multiple>
                                    @foreach ($load_types as $load_type)
                                        <option value="{{ $load_type->id }}" @if (in_array($load_type->id, old('load_type_id', $carrierPreference->load_type_id ?? []))) selected @endif>
                                            {{ $load_type->name }}
                                        </option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6 select2-primary">
                                <label for="country_id">Service Country:</label>
                                <div class="form-floating form-floating-outline">
                                    <select id="country_id" class="select2 form-select" name="country_id[]" multiple>
                                        @foreach ($countries as $country)
                                            <option value="{{ $country->id }}" @if (in_array($country->id, old('country_id', $carrierPreference->country_id ?? []))) selected @endif>
                                                {{ $country->name }}
                                            </option>
                                        @endforeach
                                    </select>
                                </div>
                            </div>

                            <!-- City Selection (Multiple) -->
                            <div class="col-md-6 select2-primary">
                                <label for="city_id">Service City</label>
                                <div class="form-floating form-floating-outline">
                                    <select id="city_id" name="city_id[]" class="form-select select2" multiple>
                                        @if ($cities != null)
                                            @foreach ($cities as $city)
                                                <option value="{{ $city->id }}" @if (in_array($city->id, old('city_id', $carrierPreference->city_id ?? []))) selected @endif>
                                                    {{ $city->name }}
                                                </option>
                                            @endforeach
                                        @endif
                                    </select>
                                </div>
                            </div>
                            <div class="col-md-6 select2-primary">
                                <label for="availability_days">Availability Days:</label>
                                <div class="form-floating form-floating-outline">
                                    <select id="availability_days" class="select2 form-select" name="availability_days[]"
                                        multiple>
                                        <option value="monday" @if (in_array('monday', old('availability_days', $carrierPreference->availability_days ?? []))) selected @endif>Monday
                                        </option>
                                        <option value="tuesday" @if (in_array('tuesday', old('availability_days', $carrierPreference->availability_days ?? []))) selected @endif>Tuesday
                                        </option>
                                        <option value="wednesday" @if (in_array('wednesday', old('availability_days', $carrierPreference->availability_days ?? []))) selected @endif>
                                            Wednesday</option>
                                        <option value="thursday" @if (in_array('thursday', old('availability_days', $carrierPreference->availability_days ?? []))) selected @endif>
                                            Thursday</option>
                                        <option value="friday" @if (in_array('friday', old('availability_days', $carrierPreference->availability_days ?? []))) selected @endif>Friday
                                        </option>
                                        <option value="saturday" @if (in_array('saturday', old('availability_days', $carrierPreference->availability_days ?? []))) selected @endif>
                                            Saturday</option>
                                        <option value="sunday" @if (in_array('sunday', old('availability_days', $carrierPreference->availability_days ?? []))) selected @endif>Sunday
                                        </option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="text-end">
                            <button type="submit" class="btn btn-primary btn-sm px-4" id="save-button">Save
                                Preferences</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    <!-- Matched Info Modal -->
    <div class="modal fade" id="aiDebugModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 640px;">
            <div class="modal-content ai-modal shadow-lg border-0 rounded-4 overflow-hidden">
                <div class="ai-gradient-bar"></div>

                <div class="modal-header border-0 pb-0">
                    <h5 class="modal-title fw-semibold">
                        AI Match Analysis <span class="text-muted" id="aiLegLabel"></span>
                    </h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>

                <div class="modal-body pt-3">
                    <!-- Generating state -->
                    <div id="aiGenerating" class="ai-generating">
                        <div class="ai-chip mb-3">
                            <span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                            GENERATING…
                        </div>
                        <div class="skeleton-line"></div>
                        <div class="skeleton-line"></div>
                        <div class="skeleton-line short"></div>
                    </div>

                    <!-- Final content -->
                    <div id="aiContent" class="d-none">
                        <ul class="list-unstyled mb-0" id="aiDebugList"></ul>
                    </div>
                </div>

                <div class="modal-footer border-0 pt-0">
                    <button type="button" class="btn btn-primary" data-bs-dismiss="modal">Close</button>
                </div>
            </div>
        </div>
    </div>

    <script src="https://cdn.sheetjs.com/xlsx-0.19.3/package/dist/xlsx.full.min.js"></script>
    <script src="https://js.stripe.com/v3/"></script>
    <script src="{{ url('assets/js/bootstrap/bootstrap.bundle.min.js') }}"></script>

    <script>
        (() => {
            // ------------- Setup -------------
            const stripe = Stripe("{{ config('services.stripe.public') }}");
            let elements = null;
            let card = null;

            // Modal elements
            const payModalEl = document.getElementById('payModal');
            const payModal = new bootstrap.Modal(payModalEl, { backdrop: 'static' });
            const legLabelEl = document.getElementById('payModalLegLabel');
            const amountLabelEl = document.getElementById('payModalAmountLabel');
            const errorEl = document.getElementById('payError');
            const successEl = document.getElementById('paySuccess');
            const confirmBtn = document.getElementById('payConfirmBtn');

            // Context for current payment
            let currentLegId = null;
            let currentClientSecret = null;

            // ------------- Helpers -------------
            function fmtUSD(n) {
                try {
                    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(Number(n));
                } catch {
                    return '$' + Number(n).toFixed(2);
                }
            }

            function showError(msg) {
                errorEl.textContent = msg || 'Payment error. Please try again.';
                errorEl.classList.remove('d-none');
                successEl.classList.add('d-none');
            }
            function showSuccess(msg) {
                successEl.textContent = msg || 'Payment succeeded!';
                successEl.classList.remove('d-none');
                errorEl.classList.add('d-none');
            }
            function clearAlerts() {
                errorEl.classList.add('d-none');
                successEl.classList.add('d-none');
                errorEl.textContent = '';
                successEl.textContent = '';
            }

            function ensureCardMounted() {
                if (!elements) elements = stripe.elements();
                if (!card) {
                    card = elements.create('card');
                    card.mount('#card-element');
                }
            }

            function setBusy(busy) {
                confirmBtn.disabled = !!busy;
                confirmBtn.innerHTML = busy
                    ? '<span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>Processing…'
                    : 'Pay now';
            }

            // ------------- Open modal from row button -------------
            document.querySelectorAll('.open-pay-modal').forEach(btn => {
                btn.addEventListener('click', async () => {
                    clearAlerts();
                    setBusy(false);

                    const legId = btn.dataset.legId;
                    const legCode = btn.dataset.legCode || '';
                    const amount = btn.dataset.amount || 0;
                    const url = btn.dataset.fundUrl;

                    currentLegId = legId;
                    currentClientSecret = null;

                    legLabelEl.textContent = legCode ? `· ${legCode}` : '';
                    amountLabelEl.textContent = fmtUSD(amount);

                    ensureCardMounted();
                    payModal.show();

                    // 1) Ask backend to create PaymentIntent and return clientSecret
                    try {
                        const res = await fetch(url, {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                                // If your /api is session-authenticated, CSRF helps (web.php). If you're using api.php with Sanctum token, replace with Authorization header.
                                'X-CSRF-TOKEN': document.querySelector('meta[name="csrf-token"]').getAttribute('content'),
                            },
                            body: JSON.stringify({})
                        });

                        const data = await res.json();
                        if (!res.ok || !data?.clientSecret) {
                            showError(data?.message || 'Could not initialize payment.');
                            return;
                        }
                        currentClientSecret = data.clientSecret;
                    } catch (e) {
                        showError('Network error while creating payment. Try again.');
                        console.error(e);
                    }
                });
            });

            // ------------- Confirm payment -------------
            confirmBtn.addEventListener('click', async () => {
                clearAlerts();

                if (!currentClientSecret || !card) {
                    showError('Payment not initialized. Close and reopen the modal.');
                    return;
                }

                setBusy(true);
                try {
                    const { error, paymentIntent } = await stripe.confirmCardPayment(currentClientSecret, {
                        payment_method: { card }
                    });

                    if (error) {
                        showError(error.message);
                        setBusy(false);
                        return;
                    }

                    if (paymentIntent && paymentIntent.status === 'succeeded') {
                        showSuccess('Payment succeeded! Funds are now held in escrow.');
                        // Give the webhook a moment to update your DB, then refresh UI
                        setTimeout(() => {
                            payModal.hide();
                            window.location.reload();
                        }, 1000);
                    } else {
                        showError('Payment did not complete. Please try again.');
                        setBusy(false);
                    }
                } catch (e) {
                    console.error(e);
                    showError('Unexpected error. Please try again.');
                    setBusy(false);
                }
            });

            // ------------- Cleanup on hide (optional keep card mounted for reuse) -------------
            payModalEl.addEventListener('hidden.bs.modal', () => {
                clearAlerts();
                setBusy(false);
                // Keep card element mounted for faster subsequent payments
            });
        })();
    </script>


    <script>
        (function () {
            const modalEl = document.getElementById('confirmFixedModal');
            const form = document.getElementById('confirmFixedForm');
            const amountLabel = document.getElementById('fixedAmountLabel');
            const legLabel = document.getElementById('fixedLegLabel');
            const amountInput = document.getElementById('fixedAmountInput');
            const confirmBtn = document.getElementById('confirmFixedBtn');

            const reCmodalEl = document.getElementById('reCconfirmFixedModal');
            const reCform = document.getElementById('reCconfirmFixedForm');
            const reCamountLabel = document.getElementById('reCfixedAmountLabel');
            const reClegLabel = document.getElementById('reCfixedLegLabel');
            const reCamountInput = document.getElementById('reCfixedAmountInput');
            const reCconfirmBtn = document.getElementById('reCconfirmFixedBtn');

            function fmtUSD(n, digits = 0) {
                try {
                    return new Intl.NumberFormat('en-US', {
                        style: 'currency',
                        currency: 'USD',
                        maximumFractionDigits: digits
                    }).format(Number(n));
                } catch {
                    return '$' + Number(n).toFixed(digits);
                }
            }

            modalEl?.addEventListener('show.bs.modal', (ev) => {
                const btn = ev.relatedTarget;
                const url = btn?.getAttribute('data-book-url') || '#';
                const amount = btn?.getAttribute('data-amount') || '0';
                const legCode = btn?.getAttribute('data-leg-code') || '';

                form.setAttribute('action', url);
                amountLabel.textContent = fmtUSD(amount, 0);
                amountInput.value = amount;
                legLabel.textContent = legCode ? `Load #${legCode}` : 'this load';

                // reset button state if previously submitted
                confirmBtn.disabled = false;
                confirmBtn.innerHTML = 'Proceed';
            });

            reCmodalEl?.addEventListener('show.bs.modal', (ev) => {
                const btnrec = ev.relatedTarget;
                const urlrec = btnrec?.getAttribute('data-book-url') || '#';
                const amountrec = btnrec?.getAttribute('data-amount') || '0';
                const legCoderec = btnrec?.getAttribute('data-leg-code') || '';

                reCform.setAttribute('action', urlrec);
                reCamountLabel.textContent = fmtUSD(amount, 0);
                reCamountInput.value = amountrec;
                reClegLabel.textContent = legCoderec ? `Load #${legCoderec}` : 'this load';

                // reset button state if previously submitted
                reCconfirmBtn.disabled = false;
                reCconfirmBtn.innerHTML = 'Proceed';
            });

            form?.addEventListener('submit', () => {
                // simple UX: disable + spinner while posting (normal HTML POST)
                confirmBtn.disabled = true;
                confirmBtn.innerHTML =
                    '<span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>Booking...';
            });

            reCform?.addEventListener('submit', () => {
                reCconfirmBtn.disabled = true;
                reCconfirmBtn.innerHTML =
                    '<span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>Booking...';
            });
        })();
    </script>
    <script>
        document.addEventListener('DOMContentLoaded', function () {
            // -------------------------
            // Elements
            // -------------------------
            const countryEl = document.getElementById('country_id');
            const cityEl = document.getElementById('city_id');
            const equipmentOwnedEl = document.getElementById('equipment_owned');
            const loadTypeEl = document.getElementById('load_type');
            const maxWeightCapacityEl = document.getElementById('max_weight_capacity');
            const availabilityDaysEl = document.getElementById('availability_days');

            // -------------------------
            // Fetch cities by country
            // -------------------------
            async function fetchCities(countryIds) {
                if (!countryIds || countryIds.length === 0) {
                    cityEl.innerHTML = '<option value="">-- Select City --</option>';
                    cityEl.disabled = true;
                    $(cityEl).trigger('change');
                    return;
                }

                cityEl.disabled = true;
                const cities = [];

                for (let countryId of countryIds) {
                    const url = "{{ url('/api/countries') }}/" + countryId + "/cities";
                    const res = await fetch(url, {
                        headers: {
                            'Accept': 'application/json'
                        }
                    });
                    const data = await res.json();
                    cities.push(...data);
                }

                cityEl.innerHTML = '';
                cities.forEach(c => {
                    const opt = document.createElement('option');
                    opt.value = c.id;
                    opt.textContent = c.name;
                    cityEl.appendChild(opt);
                });

                cityEl.disabled = false;
                $(cityEl).trigger('change');
            }

            // -------------------------
            // Initialize Select2
            // -------------------------
            $('.select2').select2();
            $(countryEl).on('change', () => fetchCities($(countryEl).val()));

            // -------------------------
            // Handle Form Submit
            // -------------------------
            document.getElementById('recommendationForm')?.addEventListener('submit', function (e) {
                e.preventDefault();

                const data = {
                    equipment_id: Array.from(equipmentOwnedEl.selectedOptions).map(o => o.value),
                    load_type_id: Array.from(loadTypeEl.selectedOptions).map(o => o.value),
                    country_id: Array.from(countryEl.selectedOptions).map(o => o.value),
                    city_id: Array.from(cityEl.selectedOptions).map(o => o.value),
                    availability_days: Array.from(availabilityDaysEl.selectedOptions).map(o => o.value),
                    max_weight_capacity: maxWeightCapacityEl.value
                };

                const btn = document.getElementById('save-button');
                btn.innerHTML = 'Saving...';
                btn.disabled = true;

                fetch('{{ route('savePreferences') }}', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-CSRF-TOKEN': '{{ csrf_token() }}'
                    },
                    body: JSON.stringify(data)
                })
                    .then(res => res.json())
                    .then(data => {
                        if (data.success) {
                            bootstrap.Modal.getInstance(document.getElementById('recommendationModal'))
                                .hide();
                            Swal.fire({
                                toast: true,
                                position: 'top-end',
                                icon: 'success',
                                title: 'Success',
                                text: 'Preferences saved successfully',
                                showConfirmButton: false,
                                timer: 2500
                            });
                        } else {
                            Swal.fire({
                                position: 'center',
                                icon: 'error',
                                title: 'Error',
                                text: data.message || 'Error submitting the form. Try again.',
                                showCloseButton: true,
                                allowOutsideClick: false,
                                allowEscapeKey: false
                            });
                        }
                    })
                    .catch(err => {
                        console.error('AJAX error:', err);
                        Swal.fire({
                            position: 'center',
                            icon: 'error',
                            title: 'Error',
                            text: 'There was an error submitting the form. Please try again.',
                            showCloseButton: true,
                            allowOutsideClick: false,
                            allowEscapeKey: false
                        });
                    })
                    .finally(() => {
                        btn.innerHTML = 'Save Preferences';
                        btn.disabled = false;
                    });
            });

            // -------------------------
            // Tooltip Init
            // -------------------------
            [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'))
                .forEach(el => new bootstrap.Tooltip(el));

            // -------------------------
            // Pagination Tabs
            // -------------------------
            window.switchTab = function (btn, tabType) {
                document.querySelectorAll('.btn-outline-light').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');

                document.querySelectorAll('.tab-pane').forEach(tab => tab.classList.remove('show', 'active'));
                document.getElementById(`tab-${tabType}`).classList.add('show', 'active');

                document.getElementById('resetPrefsBtn').classList.toggle('d-none', tabType !== 'recommended');
            };

            // -------------------------
            // Export to Excel
            // -------------------------
            window.exportToExcel = function () {
                const workbook = XLSX.utils.book_new();
                const table = document.getElementById('user-approval-table');
                const worksheet = XLSX.utils.table_to_sheet(table);
                XLSX.utils.book_append_sheet(workbook, worksheet, "Loads");
                XLSX.writeFile(workbook, 'Loads_List.xlsx');
            };

            // -------------------------
            // AI Debug Modal (Match Info)
            // -------------------------
            const modal = document.getElementById('aiDebugModal');
            const list = document.getElementById('aiDebugList');
            const gen = document.getElementById('aiGenerating');
            const body = document.getElementById('aiContent');
            const label = document.getElementById('aiLegLabel');

            document.querySelectorAll('.show-ai-debug').forEach(btn => {
                btn.addEventListener('click', () => {
                    list.innerHTML = '';
                    body.classList.add('d-none');
                    gen.classList.remove('d-none');

                    label.textContent = btn.getAttribute('data-leg') ?
                        `· ${btn.getAttribute('data-leg')}` : '';

                    let items = [];
                    try {
                        const parsed = JSON.parse(btn.getAttribute('data-debug') || '[]');
                        items = Array.isArray(parsed) ? parsed : [parsed];
                    } catch {
                        items = ['(No debug details available)'];
                    }

                    setTimeout(() => {
                        items.filter(Boolean).forEach(text => {
                            const li = document.createElement('li');
                            li.textContent = text;
                            list.appendChild(li);
                        });
                        gen.classList.add('d-none');
                        body.classList.remove('d-none');
                    }, 800);
                });
            });

            modal.addEventListener('hidden.bs.modal', () => {
                list.innerHTML = '';
                label.textContent = '';
                body.classList.add('d-none');
                gen.classList.remove('d-none');
            });
        });
    </script>

@endsection