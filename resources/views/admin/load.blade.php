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
                                                            <button class="btn btn-outline-primary btn-sm fix-width">
                                                                ${{ number_format($load_leg->price, 0) }}
                                                            </button>
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
                                                @foreach ($pending_load_legs as $i => $load_leg)
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
                                                            <button class="btn btn-outline-primary btn-sm fix-width">
                                                                ${{ number_format($load_leg->price, 0) }}
                                                            </button>
                                                        </td>
                                                        <td>
                                                            <span
                                                                class="badge rounded-pill badge-light-warning p-2">Pending</span>
                                                        </td>
                                                        <td>
                                                            <a href="{{ route('admin.loads.view', $load_leg->load_master->id) }}"
                                                                class="btn btn-sm btn-outline-primary px-3">
                                                                View
                                                            </a>
                                                            <button type="button" data-bs-toggle="modal"
                                                                data-bs-target="#updateStatus-{{ $load_leg->load_master->id }}"
                                                                class="btn btn-primary d-flex align-items-center">
                                                                <i class="mdi mdi-cog-outline mdi-20px me-1"></i> Action
                                                            </button>
                                                        </td>
                                                    </tr>
                                                @endforeach
                                            </tbody>
                                        </table>
                                        @foreach ($pending_load_legs as $i => $load_leg)
                                            <div class="modal fade" id="updateStatus-{{ $load_leg->load_master->id }}" tabindex="-1"
                                                aria-hidden="true">
                                                <div
                                                    class="modal-dialog modal-lg modal-simple modal-enable-otp modal-dialog-centered">
                                                    <div class="p-3 modal-content p-md-5">
                                                        <div class="py-3 modal-body py-md-0">
                                                            <button type="button" class="btn-close" data-bs-dismiss="modal"
                                                                aria-label="Close"></button>
                                                            <div class="mb-4 text-center">
                                                                <h3 class="mb-2">Load Forwarding</h3>
                                                            </div>
                                                            <form method="POST" class="row g-4"
                                                                action="{{ route('load.update-status', $load_leg->load_master->id) }}">
                                                                @csrf
                                                                <div class="col-12 col-md-12">
                                                                    <div class="form-floating form-floating-outline">
                                                                        <textarea class="form-control h-px-100" name="remarks" id="remarks" placeholder="Enter Remarks here..."></textarea>
                                                                        <label for="remarks">Remarks</label>
                                                                    </div>
                                                                </div>
                                                                <div class="text-center col-12">
                                                                    <button type="submit"
                                                                        class="btn btn-danger me-sm-3 me-1" name="status"
                                                                        value="7">
                                                                        Send Back
                                                                    </button>
                                                                    <button type="submit"
                                                                        class="btn btn-primary me-sm-3 me-1"
                                                                        name="status" value="2">
                                                                        Approved
                                                                    </button>
                                                                    <button type="submit"
                                                                        class="btn btn-danger me-sm-3 me-1" name="status"
                                                                        value="0">
                                                                        Reject
                                                                    </button>
                                                                </div>
                                                            </form>
                                                        </div>
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
    <script src="https://cdn.sheetjs.com/xlsx-0.19.3/package/dist/xlsx.full.min.js"></script>
    <script>
        document.addEventListener('DOMContentLoaded', function() {
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
        });
    </script>
@endsection
