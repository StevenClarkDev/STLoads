@extends('layout.app')
@section('content')
    <div class="container-fluid">
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
    <div class="container-fluid">
        <div class="row">
            <div class="col-sm-12">
                <div class="card">
                    <div class="card-body p-0">
                        <div class="mx-3">
                            <div class="card-header pb-0 card-no-border">
                                <div class="d-flex justify-content-between align-items-center flex-wrap">
                                    <div class="mb-2">
                                        <h4 class="mb-1">Loads List</h4>
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
                                                    <th>Bid Status</th>
                                                    <th>Amount</th>
                                                    <th>Payment</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                @foreach ($load_legs as $i => $load_leg)
                                                    <tr>
                                                        <td>{{ $load_leg->leg_code }}</td>
                                                        <td>
                                                            <span class="badge rounded-circle p-2 badge-primary"
                                                                data-bs-toggle="tooltip" data-bs-placement="top"
                                                                title="{{ $load_leg->pickupLocation?->name }} - {{ $load_leg->pickupLocation?->city->name }} - {{ $load_leg->pickupLocation?->country?->name }}">
                                                                <i data-feather="map-pin"></i>
                                                            </span>
                                                        </td>
                                                        <td>
                                                            <span class="badge rounded-circle p-2 badge-primary"
                                                                data-bs-toggle="tooltip" data-bs-placement="top"
                                                                title="{{ $load_leg->deliveryLocation?->name }} - {{ $load_leg->deliveryLocation?->city->name }} - {{ $load_leg->deliveryLocation?->country?->name }}">
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
                                                                    data-bs-toggle="modal"
                                                                    data-bs-target="#confirmFixedModal"
                                                                    data-book-url="{{ route('load-legs.book', $load_leg) }}"
                                                                    data-amount="{{ $load_leg->price }}"
                                                                    data-leg-code="{{ $load_leg->leg_code ?? '' }}">
                                                                    ${{ number_format($load_leg->price, 0) }}
                                                                </button>
                                                            @else
                                                                <button class="btn btn-outline-primary btn-sm fix-width"
                                                                    data-bs-toggle="modal"
                                                                    data-bs-target="#bidModal-{{ $i }}">
                                                                    ${{ number_format($load_leg->price, 0) }}
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
                                                                            <button class="btn btn-primary"
                                                                                id="confirmFixedBtn" type="submit">
                                                                                Proceed
                                                                            </button>
                                                                        </div>
                                                                    </form>
                                                                </div>
                                                            </div>


                                                            <!-- Bid Modal -->
                                                            <div class="modal fade" id="bidModal-{{ $i }}"
                                                                tabindex="-1" aria-hidden="true">
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
                                                                                        <input type="number"
                                                                                            min="1" step="1"
                                                                                            name="amount"
                                                                                            class="form-control"
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
                                                        <td>
                                                            <span
                                                                class="badge rounded-pill badge-light-warning p-2">Pending</span>
                                                        </td>
                                                    </tr>
                                                @endforeach
                                            </tbody>
                                        </table>
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
                                                    <th>Payment</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                @if ($recommendedLoadLegs != null)
                                                    @foreach ($recommendedLoadLegs as $i => $load_leg)
                                                        <tr>
                                                            <td>{{ $load_leg->leg_code }}</td>
                                                            <td>
                                                                <span class="badge rounded-circle p-2 badge-primary"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $load_leg->pickupLocation?->name }} - {{ $load_leg->pickupLocation?->city->name }} - {{ $load_leg->pickupLocation?->country?->name }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                            </td>
                                                            <td>
                                                                <span class="badge rounded-circle p-2 badge-primary"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $load_leg->deliveryLocation?->name }} - {{ $load_leg->deliveryLocation?->city->name }} - {{ $load_leg->deliveryLocation?->country?->name }}">
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
                                                            <!-- <td class="text-center">{{ $load_leg->score }}</td> -->
                                                            <!-- <td>
                                                                                                    <button class="btn btn-link toggle-debug"
                                                                                                        data-bs-toggle="collapse"
                                                                                                        data-bs-target="#debug-info-{{ $i }}"
                                                                                                        aria-expanded="false"
                                                                                                        aria-controls="debug-info-{{ $i }}">
                                                                                                        View Match Info
                                                                                                    </button>
                                                                                                    <div id="debug-info-{{ $i }}"
                                                                                                        class="collapse mt-2">
                                                                                                        @foreach ($load_leg->debug_info as $debug)
    <div>{{ $debug }}</div>
    @endforeach
                                                                                                    </div>
                                                                                                </td> -->
                                                            <!-- <td>
                                                                                                    <button
                                                                                                        class="btn btn-link show-ai-debug"
                                                                                                        data-leg="{{ $load_leg->leg_code }}"
                                                                                                        data-debug='@json($load_leg->debug_info)'
                                                                                                        data-bs-toggle="modal"
                                                                                                        data-bs-target="#aiDebugModal">
                                                                                                        View Match Info
                                                                                                    </button>
                                                                                                </td> -->
                                                            <td>
                                                                <button
                                                                    class="btn btn-sm show-ai-debug btn-outline-primary px-3 rounded-pill shadow-sm"
                                                                    data-leg="{{ $load_leg->leg_code }}"
                                                                    data-debug='@json($load_leg->debug_info)'
                                                                    data-bs-toggle="modal" data-bs-target="#aiDebugModal">
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
                                                                        data-bs-toggle="modal"
                                                                        data-bs-target="#reCconfirmFixedModal"
                                                                        data-book-url="{{ route('load-legs.book', $load_leg) }}"
                                                                        data-amount="{{ $load_leg->price }}"
                                                                        data-leg-code="{{ $load_leg->leg_code ?? '' }}">
                                                                        ${{ number_format($load_leg->price, 0) }}
                                                                    </button>
                                                                @else
                                                                    <button
                                                                        class="btn btn-outline-primary btn-sm fix-width"
                                                                        data-bs-toggle="modal"
                                                                        data-bs-target="#bidrecModal-{{ $i }}">
                                                                        ${{ number_format($load_leg->price, 0) }}
                                                                    </button>
                                                                @endif

                                                                <div class="modal fade" id="reCconfirmFixedModal"
                                                                    tabindex="-1" aria-hidden="true">
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
                                                                                    at <strong
                                                                                        id="reCfixedAmountLabel"></strong>.
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
                                                                                <button class="btn btn-light"
                                                                                    type="button"
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
                                                                <div class="modal fade"
                                                                    id="bidrecModal-{{ $i }}" tabindex="-1"
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
                                                                                            <label
                                                                                                class="form-label">Client
                                                                                                Price</label>
                                                                                            <input class="form-control"
                                                                                                value="${{ number_format($load_leg->price, 0) }}"
                                                                                                readonly>
                                                                                        </div>
                                                                                        <div class="col-md-6">
                                                                                            <label class="form-label">Your
                                                                                                Bid</label>
                                                                                            <input type="number"
                                                                                                min="1"
                                                                                                step="1"
                                                                                                name="amount"
                                                                                                class="form-control"
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
                                                            <td>
                                                                <span
                                                                    class="badge rounded-pill badge-light-warning p-2">Pending</span>
                                                            </td>
                                                        </tr>
                                                    @endforeach
                                                @elseif (!$user->carrierPreference)
                                                    <tr>
                                                        <td colspan="14"> Please Fill the Preference Form First </td>
                                                    </tr>
                                                @else
                                                    <tr>
                                                        <td colspan="14"> No recommendation available </td>
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
                                        <option value="{{ $equipment->id }}"
                                            @if (in_array($equipment->id, old('equipment_id', $carrierPreference->equipment_id ?? []))) selected @endif>
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
                                        <option value="{{ $load_type->id }}"
                                            @if (in_array($load_type->id, old('load_type_id', $carrierPreference->load_type_id ?? []))) selected @endif>
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
                                            <option value="{{ $country->id }}"
                                                @if (in_array($country->id, old('country_id', $carrierPreference->country_id ?? []))) selected @endif>
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
                                                <option value="{{ $city->id }}"
                                                    @if (in_array($city->id, old('city_id', $carrierPreference->city_id ?? []))) selected @endif>
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
    <script>
        (function() {
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
        document.addEventListener('DOMContentLoaded', function() {
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
            document.getElementById('recommendationForm')?.addEventListener('submit', function(e) {
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
            window.switchTab = function(btn, tabType) {
                document.querySelectorAll('.btn-outline-light').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');

                document.querySelectorAll('.tab-pane').forEach(tab => tab.classList.remove('show', 'active'));
                document.getElementById(`tab-${tabType}`).classList.add('show', 'active');

                document.getElementById('resetPrefsBtn').classList.toggle('d-none', tabType !== 'recommended');
            };

            // -------------------------
            // Export to Excel
            // -------------------------
            window.exportToExcel = function() {
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

    <!-- <script>
        document.addEventListener('DOMContentLoaded', function() {
            const countryEl = document.getElementById('country_id');
            const cityEl = document.getElementById('city_id');
            const equipmentOwnedEl = document.getElementById('equipment_owned');
            const loadTypeEl = document.getElementById('load_type');
            const maxWeightCapacityEl = document.getElementById('max_weight_capacity');
            const availabilityDaysEl = document.getElementById('availability_days');

            // Fetch cities based on selected countries
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
                    cities.push(...data); // Merge cities from multiple countries
                }

                // Populate city select options
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

            // Initialize Select2
            $('.select2').select2();

            // Country change -> fetch cities
            $(countryEl).on('change', () => {
                const selectedCountries = $(countryEl).val(); // array of selected country IDs
                fetchCities(selectedCountries);
            });

            // Handle form submission with AJAX
            document.getElementById('recommendationForm')?.addEventListener('submit', function(e) {
                e.preventDefault();

                // Collect the form data manually by their IDs
                const equipmentOwned = Array.from(equipmentOwnedEl.selectedOptions).map(option => option
                    .value);
                const loadType = Array.from(loadTypeEl.selectedOptions).map(option => option.value);
                const countryIds = Array.from(countryEl.selectedOptions).map(option => option.value);
                const cityIds = Array.from(cityEl.selectedOptions).map(option => option.value);
                const availabilityDays = Array.from(availabilityDaysEl.selectedOptions).map(option => option
                    .value);
                const maxWeightCapacity = maxWeightCapacityEl.value;

                // Prepare the data for submission
                const data = {
                    equipment_id: equipmentOwned,
                    load_type_id: loadType,
                    country_id: countryIds,
                    city_id: cityIds,
                    availability_days: availabilityDays,
                    max_weight_capacity: maxWeightCapacity
                };

                // Show loading indicator (optional)
                document.getElementById('save-button').innerHTML = 'Saving...';
                document.getElementById('save-button').disabled = true;

                // AJAX request to send form data to the backend
                fetch('{{ route('savePreferences') }}', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'X-CSRF-TOKEN': '{{ csrf_token() }}' // Ensure CSRF token is included in the request
                        },
                        body: JSON.stringify(data) // Convert form data to JSON
                    })
                    .then(response => response.json()) // Parse JSON response
                    .then(data => {
                        if (data.success) {
                            // Handle success - Hide modal, show success message
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
                            // Handle error (invalid data, server issues, etc.)
                            Swal.fire({
                                position: 'center',
                                icon: 'error',
                                title: 'Error',
                                text: data.message ||
                                    'There was an error submitting the form. Please try again.',
                                showConfirmButton: false,
                                showCloseButton: true,
                                allowOutsideClick: false,
                                allowEscapeKey: false,
                                backdrop: true,
                            });
                        }
                    })
                    .catch(error => {
                        Swal.fire({
                            position: 'center',
                            icon: 'error',
                            title: 'Error',
                            text: 'There was an error submitting the form. Please try again.',
                            showConfirmButton: false,
                            showCloseButton: true,
                            allowOutsideClick: false,
                            allowEscapeKey: false,
                            backdrop: true,
                        });
                        console.error('AJAX error:', error);
                    })
                    .finally(() => {
                        // Reset the button text and enable it again
                        document.getElementById('save-button').innerHTML = 'Save Preferences';
                        document.getElementById('save-button').disabled = false;
                    });
            });
        });
    </script>
                                        <script>
                                            document.querySelectorAll('.toggle-debug').forEach(item => {
                                                item.addEventListener('click', function() {
                                                    const target = document.querySelector(this.getAttribute('data-bs-target'));
                                                    target.classList.toggle('collapse');
                                                });
                                            });
                                        </script>
                                        <script>
                                            // Pagination functionality
                                            document.addEventListener('DOMContentLoaded', function() {

                                                // Update switchTab function to handle pagination
                                                window.switchTab = function(btn, tabType) {
                                                    document.querySelectorAll('.btn-outline-light').forEach(b => b.classList.remove('active'));
                                                    btn.classList.add('active');

                                                    const allTabs = document.querySelectorAll('.tab-pane');
                                                    allTabs.forEach(tab => {
                                                        tab.classList.remove('show', 'active');
                                                    });

                                                    // Show the selected tab
                                                    const selectedTab = document.getElementById(`tab-${tabType}`);
                                                    selectedTab.classList.add('show', 'active');


                                                    // Toggle reset preferences button
                                                    const resetBtn = document.getElementById('resetPrefsBtn');
                                                    resetBtn.classList.toggle('d-none', tabType !== 'recommended');
                                                };
                                            });

                                            function exportToExcel() {
                                                // Create a workbook
                                                const workbook = XLSX.utils.book_new();

                                                // Get the table
                                                const table = document.getElementById('user-approval-table');

                                                // Convert table to worksheet
                                                const worksheet = XLSX.utils.table_to_sheet(table);

                                                // Add worksheet to workbook
                                                XLSX.utils.book_append_sheet(workbook, worksheet, "Loads");

                                                // Generate Excel file and download
                                                XLSX.writeFile(workbook, 'Loads_List.xlsx');
                                            }



                                            document.addEventListener('DOMContentLoaded', function() {
                                                var tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'));
                                                tooltipTriggerList.forEach(function(tooltipTriggerEl) {
                                                    new bootstrap.Tooltip(tooltipTriggerEl);
                                                });
                                            });
                                            let recommendationPrefsExist = false; // simulate backend check

                                            // Handle form submission (only frontend)
                                            //document.getElementById('recommendationForm')?.addEventListener('submit', function(e) {
                                            //  e.preventDefault();
                                            // Normally this data would be sent to the server
                                            //const formData = Object.fromEntries(new FormData(this));
                                            //console.log('Preferences Saved:', formData);
                                            //bootstrap.Modal.getInstance(document.getElementById('recommendationModal')).hide();
                                            //});
                                        </script>
                                        <script src="https://cdn.sheetjs.com/xlsx-0.19.3/package/dist/xlsx.full.min.js"></script> -->


    <!-- <style>
                                            .btn-outline-light.active {
                                                background-color: #4d6b8a !important;
                                                color: white !important;
                                            }

                                            #collapseProduct .form-label {
                                                font-weight: 500;
                                            }

                                            #collapseProduct .form-control,
                                            #collapseProduct .form-select {
                                                font-size: 0.85rem;
                                                padding: 0.4rem 0.6rem;
                                            }

                                            .fix-width {
                                                width: 100px;
                                                text-align: center;
                                                padding: 6px 0;
                                                display: flex;
                                                justify-content: center;
                                                align-items: center;
                                            }

                                            .btn-outline-primary.fix-width:hover,
                                            .btn-outline-danger.fix-width:hover {
                                                background-color: inherit !important;
                                                color: inherit !important;
                                                border-color: inherit !important;
                                                box-shadow: none !important;
                                                transition: none !important;
                                            }

                                            /* #resetPrefsBtn {
                                                        height: 30px;
                                                        width: 30px;
                                                        padding: 0;
                                                        display: flex;
                                                        justify-content: center;
                                                        align-items: center;
                                                    } */

                                            /* Pagination styles */
                                            .pagination {
                                                margin: 0;
                                            }

                                            .pagination-circle .page-item {
                                                margin: 0 3px;
                                            }

                                            .pagination-circle .page-link {
                                                width: 32px;
                                                height: 32px;
                                                padding: 0;
                                                display: flex;
                                                align-items: center;
                                                justify-content: center;
                                                border-radius: 50% !important;
                                                border: 1px solid #dee2e6;
                                            }

                                            .pagination-circle .page-item.active .page-link {
                                                background-color: #4d6b8a;
                                                border-color: #4d6b8a;
                                            }

                                            .pagination-circle .page-item.disabled .page-link {
                                                color: #6c757d;
                                            }
                                        </style> -->
@endsection
