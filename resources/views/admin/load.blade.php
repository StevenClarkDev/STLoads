@extends('admin-layout.app')
@section('content')
    <div class="container-fluid">
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
                                    <div class="mb-2" style="min-width: 300px;">
                                        <input type="text" id="searchManageLoads" class="form-control form-control-sm" placeholder="Search by Load ID, Customer, Carrier, Status...">
                                    </div>
                                </div>
                            </div>
                            <div class="card-body">
                                <div class="list-product-header">
                                    <div>
                                        <button type="button" class="btn btn-sm btn-outline-light rounded-4 border active"
                                            onclick="switchTab(this, 'all')">All Loads ({{ $loadCount }})</button>
                                        <button type="button" class="btn btn-sm btn-outline-light rounded-4 border"
                                            onclick="switchTab(this, 'pending')">Pending Loads
                                            ({{ $pendingLoadCount }})</button>
                                        <button type="button" class="btn btn-sm btn-outline-light rounded-4 border"
                                            onclick="switchTab(this, 'release-funds')">Fund Release
                                            ({{ $releasedLoadCount }})</button>
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
                                                </tr>
                                            </thead>
                                            <tbody>
                                                @if(count($load_legs) > 0)
                                                    @foreach ($load_legs as $i => $load_leg)
                                                        <tr>
                                                            <td>{{ $load_leg->leg_code }}</td>
                                                            <td>
                                                                @php
                                                                    $pickupTitle = $load_leg->pickupLocation?->name;
                                                                    if ($load_leg->pickupLocation?->city && $load_leg->pickupLocation?->country) {
                                                                        $pickupTitle = $load_leg->pickupLocation->city->name . ', ' . $load_leg->pickupLocation->country->name;
                                                                    }
                                                                @endphp
                                                                <span class="badge rounded-circle p-2 badge-primary me-1"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $load_leg->pickupLocation?->name }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                                <span class="text-nowrap">{{ $pickupTitle }}</span>
                                                            </td>
                                                            <td>
                                                                @php
                                                                    $deliveryTitle = $load_leg->deliveryLocation?->name;
                                                                    if ($load_leg->deliveryLocation?->city && $load_leg->deliveryLocation?->country) {
                                                                        $deliveryTitle = $load_leg->deliveryLocation->city->name . ', ' . $load_leg->deliveryLocation->country->name;
                                                                    }
                                                                @endphp
                                                                <span class="badge rounded-circle p-2 badge-primary me-1"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $load_leg->deliveryLocation?->name }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                                <span class="text-nowrap">{{ $deliveryTitle }}</span>
                                                            </td>
                                                            <td>{{ \Carbon\Carbon::parse($load_leg->pickup_date)->format('jS M, Y') }}</td>
                                                            <td>{{ \Carbon\Carbon::parse($load_leg->delivery_date)->format('jS M, Y') }}</td>
                                                            <td>
                                                                <span class="badge rounded-pill bg-warning p-2 text-capitalize">{{ $load_leg->status_master?->name }}</span>
                                                            </td>
                                                            <td>
                                                                @if ($load_leg->bid_status == 'Fixed')
                                                                    <span class="badge rounded-pill bg-primary p-2 text-capitalize">{{ $load_leg->bid_status }}</span>
                                                                @else
                                                                    <span class="badge rounded-pill bg-info p-2 text-capitalize">{{ $load_leg->bid_status }}</span>
                                                                @endif
                                                            </td>
                                                            <td>
                                                                <button class="btn btn-outline-primary btn-sm fix-width">
                                                                    ${{ number_format($load_leg->price, 0) }}
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    @endforeach
                                                @else
                                                    <tr>
                                                        <td colspan="9" class="text-center py-4">No loads found.</td>
                                                    </tr>
                                                @endif
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                                <div class="tab-pane fade" id="tab-pending">
                                    <div class="table-responsive">
                                        <table class="table table-striped align-middle text-nowrap" id="user-pending-table"
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
                                                    <th>Action</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                @if(count($pending_load_legs) > 0)
                                                    @foreach ($pending_load_legs as $i => $load_leg)
                                                        <tr>
                                                            <td>{{ $load_leg->leg_code }}</td>
                                                            <td>
                                                                @php
                                                                    $pickupTitle = $load_leg->pickupLocation?->name;
                                                                    if ($load_leg->pickupLocation?->city && $load_leg->pickupLocation?->country) {
                                                                        $pickupTitle = $load_leg->pickupLocation->city->name . ', ' . $load_leg->pickupLocation->country->name;
                                                                    }
                                                                @endphp
                                                                <span class="badge rounded-circle p-2 badge-primary me-1"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $load_leg->pickupLocation?->name }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                                <span class="text-nowrap">{{ $pickupTitle }}</span>
                                                            </td>
                                                            <td>
                                                                @php
                                                                    $deliveryTitle = $load_leg->deliveryLocation?->name;
                                                                    if ($load_leg->deliveryLocation?->city && $load_leg->deliveryLocation?->country) {
                                                                        $deliveryTitle = $load_leg->deliveryLocation->city->name . ', ' . $load_leg->deliveryLocation->country->name;
                                                                    }
                                                                @endphp
                                                                <span class="badge rounded-circle p-2 badge-primary me-1"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="{{ $load_leg->deliveryLocation?->name }}">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                                <span class="text-nowrap">{{ $deliveryTitle }}</span>
                                                            </td>
                                                            <td>{{ \Carbon\Carbon::parse($load_leg->pickup_date)->format('jS M, Y') }}</td>
                                                            <td>{{ \Carbon\Carbon::parse($load_leg->delivery_date)->format('jS M, Y') }}</td>
                                                            <td>
                                                                <span class="badge rounded-pill bg-warning p-2 text-capitalize">{{ $load_leg->status_master?->name }}</span>
                                                            </td>
                                                            <td>
                                                                @if ($load_leg->bid_status == 'Fixed')
                                                                    <span class="badge rounded-pill bg-primary p-2 text-capitalize">{{ $load_leg->bid_status }}</span>
                                                                @else
                                                                    <span class="badge rounded-pill bg-info p-2 text-capitalize">{{ $load_leg->bid_status }}</span>
                                                                @endif
                                                            </td>
                                                            <td>
                                                                <button class="btn btn-outline-primary btn-sm fix-width">
                                                                    ${{ number_format($load_leg->price, 0) }}
                                                                </button>
                                                            </td>
                                                            <td>
                                                                <span class="badge rounded-pill badge-light-warning p-2">Pending</span>
                                                            </td>
                                                            <!-- <td>
                                                                <a href="{{ route('admin.loads.view', $load_leg->load_master->id) }}"
                                                                    class="btn btn-sm btn-outline-primary px-3">
                                                                    View
                                                                </a>
                                                                <button type="button" data-bs-toggle="modal"
                                                                    data-bs-target="#updateStatus-{{ $load_leg->load_master->id }}"
                                                                    class="btn btn-primary d-flex align-items-center"></button>
                                                                    <i class="mdi mdi-cog-outline mdi-20px me-1"></i> Action
                                                                </button>
                                                            </td> -->
                                                            <td class="d-flex gap-1">
                                                <a href="{{ route('admin.loads.view', $load_leg->load_master->id) }}"
                                                    class="btn btn-info btn-sm w-80">Profile</a>
                                                <button type="button" data-bs-toggle="modal"
                                                    data-bs-target="#updateStatus-{{ $load_leg->load_master->id }}"
                                                    class="btn btn-primary btn-sm w-80">Action</button>
                                            </td>
                                                        </tr>
                                                    @endforeach
                                                @else
                                                    <tr>
                                                        <td colspan="10" class="text-center py-4">No pending loads found.</td>
                                                    </tr>
                                                @endif
                                            </tbody>
                                        </table>
                                        @foreach ($pending_load_legs as $i => $load_leg)
                                            <div class="modal fade" id="updateStatus-{{ $load_leg->load_master->id }}" tabindex="-1" aria-hidden="true">
    <div class="modal-dialog modal-md modal-dialog-centered">
        <div class="modal-content border-0 shadow-sm rounded-3">
            <div class="modal-header border-0">
                <h5 class="modal-title">Load Forwarding</h5>
                <button type="button" class="btn-close" data-bs-dismiss="modal"
                    aria-label="Close"></button>
            </div>

            <form method="POST" action="{{ route('load.update-status', $load_leg->load_master->id) }}">
                @csrf
                <div class="modal-body">
                    <div class="mb-3">
                        <label for="remarks" class="form-label fw-medium">Remarks</label>
                        <textarea class="form-control" name="remarks" id="remarks" rows="3"
                            placeholder="Enter remarks (optional for Approve, required for Reject/Send Back)"></textarea>
                    </div>
                </div>

                <div class="modal-footer border-0 d-flex justify-content-end gap-1">
                    <!-- Send Back -->
                    <button type="submit" class="btn btn-secondary btn-sm" name="status" value="7">
                        Send Back
                    </button>
                    <!-- Approve -->
                    <button type="submit" class="btn btn-primary btn-sm" name="status" value="2">
                        Approved
                    </button>
                    <!-- Reject -->
                    <button type="submit" class="btn btn-danger btn-sm" name="status" value="0">
                        Reject
                    </button>
                </div>
            </form>
        </div>
    </div>
</div>

                                        @endforeach
                                    </div>
                                </div>
                                
<div class="tab-pane fade" id="tab-release-funds">
    <div class="table-responsive">
        <table class="table table-striped align-middle text-nowrap" id="user-pending-table"
            style="font-size: 0.875rem;">
            <thead class="bg-white" style="position: sticky; top: 0; z-index: 2;">
                <tr>
                    <th>Load ID</th>
                    <th>Carrier</th>
                    <th>User</th>
                    <th>Status</th>
                    <th>Amount</th>
                    <th>Action</th>
                </tr>
            </thead>
            <tbody>
                @if(count($release_load_legs) > 0)
                    @foreach ($release_load_legs as $i => $load_leg)
                        <tr>
                            <td>{{ $load_leg->leg_code }}</td>
                            <td>{{ $load_leg->carrier?->name }}</td>
                            <td>{{ $load_leg->load_master?->user?->name }}</td>
                            <td>
                                <span class="badge rounded-pill bg-light-success p-2 text-capitalize">
                                    {{ $load_leg->status_master?->name }}
                                </span>
                            </td>
                            <td>
                                <button class="btn btn-outline-primary btn-sm fix-width">
                                    ${{ number_format($load_leg->booked_amount ?? $load_leg->price ?? 0, 0) }}
                                </button>
                            </td>
                            <td class="d-flex gap-1">
                                <a href="{{ route('admin.loads.view', $load_leg->load_master->id) }}"
                                   class="btn btn-info btn-sm w-80">
                                    Profile
                                </a>

                                {{-- Open Release Funds Modal --}}
                                <button type="button"
                                        class="btn btn-success btn-sm w-80"
                                        data-bs-toggle="modal"
                                        data-bs-target="#releaseFunds-{{ $load_leg->escrow?->id }}">
                                     Release Funds
                                </button>
                            </td>
                        </tr>
                    @endforeach
                @else
                    <tr>
                        <td colspan="10" class="text-center py-4">No funds to release.</td>
                    </tr>
                @endif
            </tbody>
        </table>

        {{-- Release Funds Modals --}}
        @foreach ($release_load_legs as $i => $load_leg)
            <div class="modal fade" id="releaseFunds-{{ $load_leg->escrow->id }}" tabindex="-1" aria-hidden="true">
                <div class="modal-dialog modal-md modal-dialog-centered">
                    <div class="modal-content border-0 shadow-sm rounded-3">
                        <div class="modal-header border-0">
                            <h5 class="modal-title">Release Funds</h5>
                            <button type="button" class="btn-close"
                                    data-bs-dismiss="modal"
                                    aria-label="Close"></button>
                        </div>

                        <form method="POST" action="{{ route('admin.escrows.release', $load_leg->escrow->id) }}">
                            @csrf
                            <div class="modal-body">
                                <p class="mb-3">
                                    Are you sure you want to release funds to this carrier?
                                </p>

                                <ul class="list-unstyled small mb-0">
                                    <li class="mb-2">
                                        <div class="row">
                                            <div class="col-6">
                                                <strong>Load ID:</strong> {{ $load_leg->leg_code }}
                                            </div>
                                            <div class="col-6">
                                                <strong>Carrier:</strong> {{ $load_leg->carrier?->name }}
                                            </div>
                                        </div>
                                    </li>
                                    <li class="mb-2">
                                        <div class="row">
                                            <div class="col-6">
                                                <strong>Amount:</strong>
                                        ${{ number_format($load_leg->booked_amount ?? $load_leg->price ?? 0, 0) }}
                                            </div>
                                            <div class="col-6">
                                                <strong>Status:</strong>
                                        <span class="badge rounded-pill bg-primary p-2 text-capitalize">{{ $load_leg->status_master?->name }}</span>
                                        
                                            </div>
                                        </div>
                                    </li>
                                </ul>
                            </div>

                            <div class="modal-footer border-0 d-flex justify-content-end">
                                <button type="button" class="btn btn-secondary btn-sm"
                                        data-bs-dismiss="modal">
                                    Cancel
                                </button>
                                <button type="submit" class="btn btn-success btn-sm">
                                    <i class="fas fa-money-bill"></i> Confirm Release
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        @endforeach
    </div>
</div>

                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <!-- Font Awesome (for the money-bill icon) -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/5.15.4/css/all.min.css">

    <script src="https://cdn.sheetjs.com/xlsx-0.19.3/package/dist/xlsx.full.min.js"></script>
    <script src="https://unpkg.com/feather-icons"></script>
    <script>
        document.addEventListener('DOMContentLoaded', function() {
            // --- Search functionality for Manage Loads ---
            const searchInput = document.getElementById('searchManageLoads');
            
            if (searchInput) {
                searchInput.addEventListener('keyup', function() {
                    const filter = this.value.toLowerCase();
                    const tables = ['user-approval-table', 'user-pending-table', 'user-release-table'];
                    
                    tables.forEach(tableId => {
                        const table = document.getElementById(tableId);
                        if (table) {
                            const rows = table.querySelectorAll('tbody tr');
                            
                            rows.forEach(row => {
                                const cells = row.querySelectorAll('td');
                                let found = false;
                                
                                cells.forEach(cell => {
                                    if (cell.textContent.toLowerCase().includes(filter)) {
                                        found = true;
                                    }
                                });
                                
                                row.style.display = found ? '' : 'none';
                            });
                        }
                    });
                });
            }

            // --- Tabs ---
            window.switchTab = function(btn, tabType) {
                // limit to these buttons only
                document.querySelectorAll('.list-product-header .btn-outline-light')
                    .forEach(b => b.classList.remove('active'));
                btn.classList.add('active');

                document.querySelectorAll('.tab-pane')
                    .forEach(tab => tab.classList.remove('show', 'active'));

                const pane = document.getElementById(`tab-${tabType}`);
                if (pane) pane.classList.add('show', 'active');
            };

            // --- Only run if a modal exists ---
            const modal = document.getElementById('verifyModal'); // or the actual modal id
            if (modal) {
                modal.addEventListener('hidden.bs.modal', () => {
                    const list = document.getElementById('list'); // if you have these ids
                    const label = document.getElementById('label');
                    const body = document.getElementById('body');
                    const gen = document.getElementById('gen');
                    if (list) list.innerHTML = '';
                    if (label) label.textContent = '';
                    if (body) body.classList.add('d-none');
                    if (gen) gen.classList.remove('d-none');
                });
            }

            // Initialize feather icons (renders elements with data-feather)
            try {
                if (window.feather && typeof window.feather.replace === 'function') {
                    window.feather.replace();
                }
            } catch (e) {
                // fail silently if feather is not available
                console.warn('Feather icons init failed', e);
            }
        });
    </script>
@endsection
