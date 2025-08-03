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
                        <div class="card mx-3">
                            <div class="card-header pb-0 card-no-border">
                                <div class="d-flex justify-content-between align-items-center flex-wrap">
                                    <div class="mb-2">
                                        <h4 class="mb-1">Loads List</h4>
                                        <span>See Registered Loads below.</span>
                                    </div>
                                    <div class="d-flex gap-2">
                                        <a href="{{ route('loads.add') }}" class="btn btn-sm btn-primary px-3" type="button">
                                            <i class="bi bi-plus-circle me-1"></i> Add Load
                                        </a>
                                        <button class="btn btn-sm btn-outline-primary px-3" type="button"
                                            data-bs-toggle="collapse" data-bs-target="#collapseProduct"
                                            aria-expanded="false" aria-controls="collapseProduct">
                                            <i class="bi bi-filter me-1"></i> Filter
                                        </button>
                                        <button class="btn btn-sm btn-outline-primary px-3" onclick="exportToExcel()">
                                            <i class="bi bi-download me-1"></i> Export
                                        </button>
                                    </div>
                                </div>
                            </div>
                            <div class="collapse" id="collapseProduct">
                                <div class="card card-body list-product-body">
                                    <div class="row ">
                                        <div class="mb-3">
                                            <label class="form-label small">Status</label>
                                            <select class="form-select form-select-sm">
                                                <option selected>All Statuses</option>
                                                <option>Pending</option>
                                                <option>Accepted</option>
                                                <option>Rejected</option>
                                            </select>
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label small">Payment</label>
                                            <select class="form-select form-select-sm">
                                                <option selected>All Payments</option>
                                                <option>Paid</option>
                                                <option>Unpaid</option>
                                                <option>Pending</option>
                                            </select>
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label small">Date Range</label>
                                            <input type="date" class="form-control form-control-sm mb-2"
                                                placeholder="From">
                                            <input type="date" class="form-control form-control-sm" placeholder="To">
                                        </div>
                                        <button class="btn btn-sm btn-primary w-100">Apply Filters</button>
                                    </div>
                                </div>
                            </div>
                            <div class="card-body">
                                <div class="list-product-header">
                                    <div>
                                        <button class="btn btn-sm btn-outline-light rounded-4 border active"
                                            onclick="switchTab(this, 'all')">All Loads (512)</button>
                                        <button class="btn btn-sm btn-outline-light rounded-4 border"
                                            onclick="switchTab(this, 'recommended')">Recommended Loads (52)</button>
                                        <button class="btn btn-sm btn-outline-light rounded-4 border"
                                            onclick="switchTab(this, 'accepted')">Accepted Loads (211)</button>
                                        <button class="btn btn-sm btn-outline-light rounded-4 border"
                                            onclick="switchTab(this, 'time')">Time-Sensitive (48)</button>
                                    </div>
                                </div>
                            </div>
                            <div class="card-body">
                                <div class="table-responsive user-datatable">
                                    <div style="max-height: 500px; min-height: 210px; overflow-y: auto;">
                                        <div class="table-responsive">
                                            <table class="table table-striped align-middle text-nowrap"
                                                id="user-approval-table" style="font-size: 0.875rem;">
                                                <thead class="bg-white" style="position: sticky; top: 0; z-index: 2;">
                                                    <tr>
                                                        <th>Load ID</th>
                                                        <th>Origin</th>
                                                        <th>Destination</th>
                                                        <th>Schedule</th>
                                                        <th>Status</th>
                                                        <th>Bid</th>
                                                        <th>Payment</th>
                                                        <th>Lane & Mode</th>
                                                        <th>Client</th>
                                                        <th>Actions</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    @foreach (range(1, 8) as $i)
                                                        <tr
                                                            class="tab-all tab-{{ $i % 3 == 0 ? 'time' : ($i % 2 == 0 ? 'accepted' : 'recommended') }}">
                                                            <td>L-27{{ 80 + $i }}</td>
                                                            <td><span class="badge rounded-circle p-2 badge-primary"><i
                                                                        data-feather="map-pin"></i></span></td>
                                                            <td><span class="badge rounded-circle p-2 badge-primary"><i
                                                                        data-feather="map-pin"></i></span></td>
                                                            <td>{{ now()->addDays($i)->format('jS M, Y') }}</td>
                                                            <td>
                                                                @php
                                                                    $status = ['accepted', 'pending', 'rejected'][
                                                                        $i % 3
                                                                    ];
                                                                @endphp
                                                                <button
                                                                    class="btn btn-{{ $status == 'pending' ? 'primary' : 'outline-' . ($status == 'accepted' ? 'primary' : 'danger') }} d-flex align-items-center px-2 py-1"
                                                                    type="button">
                                                                    {{ ucfirst($status) }}
                                                                </button>
                                                            </td>
                                                            <td>
                                                                @php
                                                                    $status = ['accepted', 'pending', 'rejected'][
                                                                        $i % 3
                                                                    ];
                                                                @endphp
                                                                <button
                                                                    class="btn btn-{{ $status == 'pending' ? 'primary' : 'outline-' . ($status == 'accepted' ? 'primary' : 'danger') }} d-flex align-items-center px-2 py-1"
                                                                    type="button" data-bs-toggle="modal"
                                                                    data-bs-target="#statusModal-{{ $i }}">
                                                                    {{ ucfirst($status) }}
                                                                </button>

                                                                <!-- Status Modal -->
                                                                <div class="modal fade" id="statusModal-{{ $i }}"
                                                                    tabindex="-1" aria-hidden="true">
                                                                    <div class="modal-dialog modal-dialog-centered"
                                                                        style="max-width: 600px;">
                                                                        <div class="modal-content p-4">
                                                                            <div class="modal-header border-0">
                                                                                <h5 class="modal-title">
                                                                                    @if ($status == 'accepted')
                                                                                        Bid Approved
                                                                                    @elseif($status == 'rejected')
                                                                                        Bid Not Approved
                                                                                    @else
                                                                                        Submit Your Bid
                                                                                    @endif
                                                                                </h5>
                                                                                <button type="button" class="btn-close"
                                                                                    data-bs-dismiss="modal"
                                                                                    aria-label="Close"></button>
                                                                            </div>

                                                                            <div class="modal-body">
                                                                                @if ($status == 'accepted')
                                                                                    <p class="text-muted mb-4">
                                                                                        Congratulations!
                                                                                        Your
                                                                                        bid has
                                                                                        been accepted by the client.</p>
                                                                                @elseif($status == 'rejected')
                                                                                    <p class="text-muted mb-4">Client
                                                                                        has
                                                                                        rejected
                                                                                        your bid
                                                                                        due to high pricing or other
                                                                                        concerns.
                                                                                    </p>
                                                                                @else
                                                                                    <p class="text-muted mb-4">Please
                                                                                        review the
                                                                                        client's
                                                                                        offer and submit your bid below.
                                                                                    </p>
                                                                                @endif

                                                                                <div class="row my-3">
                                                                                    <div class="col-md-6">
                                                                                        <label class="form-label">Client
                                                                                            Price</label>
                                                                                        <input type="text"
                                                                                            class="form-control"
                                                                                            value="$10,000" readonly>
                                                                                    </div>
                                                                                    <div class="col-md-6">
                                                                                        <label class="form-label">Your
                                                                                            Bid</label>
                                                                                        @if ($status == 'pending')
                                                                                            <input type="text"
                                                                                                class="form-control"
                                                                                                placeholder="Enter your bid">
                                                                                        @else
                                                                                            <input type="text"
                                                                                                class="form-control"
                                                                                                value="$11,000" readonly>
                                                                                        @endif
                                                                                    </div>
                                                                                </div>

                                                                                @if ($status == 'accepted')
                                                                                    <div class="row mb-3">
                                                                                        <div class="col-md-6">
                                                                                            <label class="form-label">Bid
                                                                                                Submitted
                                                                                                On</label>
                                                                                            <input type="text"
                                                                                                class="form-control"
                                                                                                value="{{ now()->subDays(2)->format('d M Y') }}"
                                                                                                readonly>
                                                                                        </div>
                                                                                        <div class="col-md-6">
                                                                                            <label
                                                                                                class="form-label">Accepted
                                                                                                On</label>
                                                                                            <input type="text"
                                                                                                class="form-control"
                                                                                                value="{{ now()->subDay()->format('d M Y') }}"
                                                                                                readonly>
                                                                                        </div>
                                                                                    </div>
                                                                                @endif

                                                                                <div
                                                                                    class="d-flex justify-content-end gap-2 mt-4">
                                                                                    <a href="{{ route('chat')}}"
                                                                                        class="btn btn-primary">Contact</a>
                                                                                </div>
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </td>
                                                            <td>
                                                                @if ($i % 4 == 0)
                                                                    <span
                                                                        class="badge rounded-pill badge badge-light-warning p-2">Pending</span>
                                                                @else
                                                                    <span
                                                                        class="badge rounded-pill badge badge-light-{{ $i % 2 == 0 ? 'success' : 'danger' }} p-2">{{ $i % 2 == 0 ? 'Paid' : 'Unpaid' }}</span>
                                                                @endif
                                                            </td>
                                                            <td><span class="badge p-2 badge-light text-dark"><i
                                                                        data-feather="map-pin"></i></span> SYD - MLB
                                                            </td>
                                                            <td>BlueShip Pvt Ltd</td>
                                                            <td>
                                                                <a href="#" title="View"
                                                                    style="color: #8a949dff;"><i
                                                                        class="bi bi-eye"></i></a>
                                                                <a href="#" title="Share"
                                                                    style="color: #8a949dff;"><i
                                                                        class="bi bi-share"></i></a>
                                                                <a href="#" title="Delete"
                                                                    style="color: #8a949dff;"><i
                                                                        class="bi bi-trash"></i></a>
                                                            </td>
                                                        </tr>
                                                    @endforeach
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>


    <script>
        function switchTab(btn, tabType) {
            document.querySelectorAll('.btn-outline-light').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');

            const rows = document.querySelectorAll('#user-approval-table tbody tr');
            rows.forEach(row => {
                if (tabType === 'all') {
                    row.style.display = '';
                } else {
                    row.style.display = row.classList.contains('tab-' + tabType) ? '' : 'none';
                }
            });
        }

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
    </script>

    <style>
        .btn-outline-light.active {
            background-color: #4d6b8a !important;
            color: white !important;
        }
    </style>

    <!-- Include SheetJS for Excel export -->
    <script src="https://cdn.sheetjs.com/xlsx-0.19.3/package/dist/xlsx.full.min.js"></script>
@endsection
