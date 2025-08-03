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
                                        <button id="resetPrefsBtn" class="btn btn-sm btn-secondary px-2 d-none"
                                            type="button" data-bs-toggle="modal" data-bs-target="#recommendationModal"
                                            title="Reset Preferences">
                                            <i class="bi bi-arrow-clockwise"></i>
                                        </button>
                                        <a href="{{ route('loads.add') }}" class="btn btn-sm btn-primary px-3"
                                            type="button">
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
                                    <form id="filterForm" onsubmit="applyFilters(event)">
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
                                                <input type="date" class="form-control form-control-sm" name="from">
                                            </div>
                                            <div class="col-md-3">
                                                <label class="form-label small">To</label>
                                                <input type="date" class="form-control form-control-sm" name="to">
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
                                                        <th>Bid Amount</th>
                                                        <th>Payment</th>
                                                        <th>Lane & Mode</th>
                                                        <th>Client</th>
                                                        <th>Actions</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    @foreach (range(1, 36) as $i)
                                                        @php
                                                            $statuses = ['available', 'booked', 'picked', 'delivered'];
                                                            $statusText = $statuses[$i % 4];
                                                            $statusClass = match ($statusText) {
                                                                'available' => 'info',
                                                                'booked' => 'primary',
                                                                'picked' => 'warning',
                                                                'delivered' => 'success',
                                                            };
                                                            $bidStatus = ['original', 'accepted', 'rejected'][$i % 3];
                                                            $tabType = match ($i % 4) {
                                                                0 => 'time',
                                                                1 => 'recommended',
                                                                2, 3 => 'accepted',
                                                            };
                                                        @endphp
                                                        <tr class="tab-all tab-{{ $tabType }}">
                                                            <td>L-27{{ 80 + $i }}</td>
                                                            <td>
                                                                <span class="badge rounded-circle p-2 badge-primary"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="128 Pitt Street, Sydney">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                            </td>
                                                            <td>
                                                                <span class="badge rounded-circle p-2 badge-primary"
                                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                                    title="42 George Street, Melbourne">
                                                                    <i data-feather="map-pin"></i>
                                                                </span>
                                                            </td>
                                                            <td>{{ now()->addDays($i)->format('jS M, Y') }}</td>
                                                            <td>
                                                                <span
                                                                    class="badge rounded-pill bg-{{ $statusClass }} p-2 text-capitalize">{{ $statusText }}</span>
                                                            </td>
                                                            <td>
                                                                @if($bidStatus == 'original')
                                                                    <button class="btn btn-primary btn-sm fix-width"
                                                                        data-bs-toggle="modal" data-bs-target="#bidModal-{{ $i }}">
                                                                        ${{ number_format(rand(8000, 15000), 0) }}
                                                                    </button>
                                                                @elseif($bidStatus == 'accepted')
                                                                    <button class="btn btn-outline-primary btn-sm fix-width"
                                                                        data-bs-toggle="modal" data-bs-target="#bidModal-{{ $i }}">
                                                                        Accepted
                                                                    </button>
                                                                @else
                                                                    <button class="btn btn-outline-danger btn-sm fix-width"
                                                                        data-bs-toggle="modal" data-bs-target="#bidModal-{{ $i }}">
                                                                        Rejected
                                                                    </button>
                                                                @endif

                                                                <!-- Bid Modal -->
                                                                <div class="modal fade" id="bidModal-{{ $i }}" tabindex="-1"
                                                                    aria-hidden="true">
                                                                    <div class="modal-dialog modal-dialog-centered"
                                                                        style="max-width: 600px;">
                                                                        <div class="modal-content p-4">
                                                                            <div class="modal-header border-0">
                                                                                <h5 class="modal-title">
                                                                                    @if ($bidStatus == 'accepted')
                                                                                        Bid Approved
                                                                                    @elseif($bidStatus == 'rejected')
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
                                                                                @if ($bidStatus == 'accepted')
                                                                                    <p class="text-muted mb-4">Congratulations! Your
                                                                                        bid has been accepted by the client.</p>
                                                                                @elseif($bidStatus == 'rejected')
                                                                                    <p class="text-muted mb-4">Client has rejected
                                                                                        your bid due to high pricing or other
                                                                                        concerns.</p>
                                                                                @else
                                                                                    <p class="text-muted mb-4">Please review the
                                                                                        client's offer and submit your bid below.
                                                                                    </p>
                                                                                @endif

                                                                                <div class="row my-3">
                                                                                    <div class="col-md-6">
                                                                                        <label class="form-label">Client
                                                                                            Price</label>
                                                                                        <input type="text" class="form-control"
                                                                                            value="${{ number_format(rand(8000, 12000), 0) }}"
                                                                                            readonly>
                                                                                    </div>
                                                                                    <div class="col-md-6">
                                                                                        <label class="form-label">Your
                                                                                            Bid</label>
                                                                                        @if ($bidStatus == 'original')
                                                                                            <input type="text" class="form-control"
                                                                                                placeholder="Enter your bid">
                                                                                        @else
                                                                                            <input type="text" class="form-control"
                                                                                                value="${{ number_format(rand(8000, 15000), 0) }}"
                                                                                                readonly>
                                                                                        @endif
                                                                                    </div>
                                                                                </div>

                                                                                @if ($bidStatus == 'accepted')
                                                                                    <div class="row mb-3">
                                                                                        <div class="col-md-6">
                                                                                            <label class="form-label">Bid Submitted
                                                                                                On</label>
                                                                                            <input type="text" class="form-control"
                                                                                                value="{{ now()->subDays(2)->format('d M Y') }}"
                                                                                                readonly>
                                                                                        </div>
                                                                                        <div class="col-md-6">
                                                                                            <label class="form-label">Accepted
                                                                                                On</label>
                                                                                            <input type="text" class="form-control"
                                                                                                value="{{ now()->subDay()->format('d M Y') }}"
                                                                                                readonly>
                                                                                        </div>
                                                                                    </div>
                                                                                @endif

                                                                                <div
                                                                                    class="d-flex justify-content-end gap-2 mt-4">
                                                                                    <a href="{{ route('chat') }}"
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
                                                                        data-feather="map-pin"></i></span> SYD - MLB</td>
                                                            <td>BlueShip Pvt Ltd</td>
                                                            <td>
                                                                <a href="#" title="View" style="color: #8a949dff;"><i
                                                                        class="bi bi-eye"></i></a>
                                                                <a href="#" title="Share" style="color: #8a949dff;"><i
                                                                        class="bi bi-share"></i></a>
                                                                <a href="#" title="Delete" style="color: #8a949dff;"><i
                                                                        class="bi bi-trash"></i></a>
                                                            </td>
                                                        </tr>
                                                    @endforeach
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                                <!-- Pagination -->
                                <div class="d-flex justify-content-end mt-3">
                                    <nav aria-label="Page navigation">
                                        <ul class="pagination pagination-sm pagination-circle mb-0">
                                            <li class="page-item disabled" id="prevPage">
                                                <a class="page-link" href="#" tabindex="-1" aria-disabled="true">
                                                    <i class="bi bi-chevron-left"></i>
                                                </a>
                                            </li>
                                            <li class="page-item active"><a class="page-link" href="#">1</a></li>
                                            <li class="page-item"><a class="page-link" href="#">2</a></li>
                                            <li class="page-item"><a class="page-link" href="#">3</a></li>
                                            <li class="page-item" id="nextPage">
                                                <a class="page-link" href="#">
                                                    <i class="bi bi-chevron-right"></i>
                                                </a>
                                            </li>
                                        </ul>
                                    </nav>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <!-- Recommendation Preferences Modal -->
    <div class="modal fade" id="recommendationModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 500px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title">Your Load Preferences</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="recommendationForm">
                        <div class="mb-3">
                            <label class="form-label">Role</label>
                            <select class="form-select form-select-sm" name="role" required>
                                <option value="">Select your role</option>
                                <option value="carrier">Carrier</option>
                                <option value="broker">Broker</option>
                                <option value="other">Other</option>
                            </select>
                        </div>
                        <div class="mb-3">
                            <label class="form-label">Preferred Lane</label>
                            <input type="text" class="form-control form-control-sm" name="lane"
                                placeholder="e.g., SYD - MLB">
                        </div>
                        <div class="mb-3">
                            <label class="form-label">Load Type</label>
                            <select class="form-select form-select-sm" name="mode">
                                <option value="">Select Mode</option>
                                <option value="FTL">FTL</option>
                                <option value="LTL">LTL</option>
                            </select>
                        </div>
                        <div class="mb-3">
                            <label class="form-label">Preferred Origin</label>
                            <input type="text" class="form-control form-control-sm" name="origin" placeholder="City or ZIP">
                        </div>
                        <div class="mb-3">
                            <label class="form-label">Preferred Destination</label>
                            <input type="text" class="form-control form-control-sm" name="destination"
                                placeholder="City or ZIP">
                        </div>
                        <div class="mb-3">
                            <label class="form-label">Max Weight (in tons)</label>
                            <input type="number" class="form-control form-control-sm" name="weight" placeholder="e.g., 20">
                        </div>
                        <div class="mb-3">
                            <label class="form-label">Available From</label>
                            <input type="date" class="form-control form-control-sm" name="available_date">
                        </div>

                        <div class="text-end">
                            <button type="submit" class="btn btn-primary btn-sm px-4">Save Preferences</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>


    <script>
        // Pagination functionality
        document.addEventListener('DOMContentLoaded', function () {
            const rowsPerPage = 12;
            let currentPage = 1;
            let currentTab = 'all';

            // Initialize pagination
            initPagination();

            function initPagination() {
                updatePagination();

                // Handle page clicks
                document.querySelectorAll('.pagination .page-link').forEach(link => {
                    link.addEventListener('click', function (e) {
                        e.preventDefault();
                        const target = e.target.closest('a');
                        if (target.querySelector('.bi-chevron-left')) {
                            if (currentPage > 1) {
                                currentPage--;
                                updatePagination();
                            }
                        } else if (target.querySelector('.bi-chevron-right')) {
                            const totalPages = Math.ceil(getVisibleRows().length / rowsPerPage);
                            if (currentPage < totalPages) {
                                currentPage++;
                                updatePagination();
                            }
                        } else if (!isNaN(target.textContent)) {
                            currentPage = parseInt(target.textContent);
                            updatePagination();
                        }
                    });
                });
            }

            function getVisibleRows() {
                if (currentTab === 'all') {
                    return Array.from(document.querySelectorAll('#user-approval-table tbody tr.tab-all'));
                } else {
                    return Array.from(document.querySelectorAll(`#user-approval-table tbody tr.tab-${currentTab}`));
                }
            }

            function updatePagination() {
                const visibleRows = getVisibleRows();
                const totalPages = Math.ceil(visibleRows.length / rowsPerPage);

                // Hide all rows
                visibleRows.forEach(row => row.style.display = 'none');

                // Show rows for current page
                const start = (currentPage - 1) * rowsPerPage;
                const end = start + rowsPerPage;
                visibleRows.slice(start, end).forEach(row => row.style.display = '');

                // Update pagination UI
                const pagination = document.querySelector('.pagination');
                pagination.innerHTML = '';

                // Previous button
                pagination.innerHTML += `
                    <li class="page-item ${currentPage === 1 ? 'disabled' : ''}" id="prevPage">
                        <a class="page-link" href="#" tabindex="-1" aria-disabled="true">
                            <i class="bi bi-chevron-left"></i>
                        </a>
                    </li>
                `;

                // Page numbers - show max 3 pages around current
                const startPage = Math.max(1, currentPage - 1);
                const endPage = Math.min(totalPages, currentPage + 1);

                if (startPage > 1) {
                    pagination.innerHTML += `
                        <li class="page-item ${1 === currentPage ? 'active' : ''}">
                            <a class="page-link" href="#">1</a>
                        </li>
                    `;
                    if (startPage > 2) {
                        pagination.innerHTML += `<li class="page-item disabled"><span class="page-link">...</span></li>`;
                    }
                }

                for (let i = startPage; i <= endPage; i++) {
                    pagination.innerHTML += `
                        <li class="page-item ${i === currentPage ? 'active' : ''}">
                            <a class="page-link" href="#">${i}</a>
                        </li>
                    `;
                }

                if (endPage < totalPages) {
                    if (endPage < totalPages - 1) {
                        pagination.innerHTML += `<li class="page-item disabled"><span class="page-link">...</span></li>`;
                    }
                    pagination.innerHTML += `
                        <li class="page-item ${totalPages === currentPage ? 'active' : ''}">
                            <a class="page-link" href="#">${totalPages}</a>
                        </li>
                    `;
                }

                // Next button
                pagination.innerHTML += `
                    <li class="page-item ${currentPage === totalPages ? 'disabled' : ''}" id="nextPage">
                        <a class="page-link" href="#">
                            <i class="bi bi-chevron-right"></i>
                        </a>
                    </li>
                `;

                // Add event listeners to new pagination buttons
                document.querySelectorAll('.pagination .page-link').forEach(link => {
                    link.addEventListener('click', function (e) {
                        e.preventDefault();
                        const target = e.target.closest('a');
                        if (target.querySelector('.bi-chevron-left')) {
                            if (currentPage > 1) {
                                currentPage--;
                                updatePagination();
                            }
                        } else if (target.querySelector('.bi-chevron-right')) {
                            if (currentPage < totalPages) {
                                currentPage++;
                                updatePagination();
                            }
                        } else if (!isNaN(target.textContent)) {
                            currentPage = parseInt(target.textContent);
                            updatePagination();
                        }
                    });
                });
            }

            // Update switchTab function to handle pagination
            window.switchTab = function (btn, tabType) {
                document.querySelectorAll('.btn-outline-light').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                currentTab = tabType;
                currentPage = 1;
                updatePagination();

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


        function applyFilters(event) {
            event.preventDefault();
            const form = document.getElementById('filterForm');
            const status = form.status.value.toLowerCase();
            const payment = form.payment.value.toLowerCase();
            const from = new Date(form.from.value);
            const to = new Date(form.to.value);

            const rows = document.querySelectorAll('#user-approval-table tbody tr');
            rows.forEach(row => {
                let show = true;

                if (status) {
                    const rowStatus = row.querySelector('td:nth-child(5) button')?.textContent?.trim().toLowerCase();
                    if (rowStatus !== status) show = false;
                }

                if (payment) {
                    const rowPayment = row.querySelector('td:nth-child(7) span')?.textContent?.trim().toLowerCase();
                    if (rowPayment !== payment) show = false;
                }

                if (form.from.value && form.to.value) {
                    const rowDateText = row.querySelector('td:nth-child(4)')?.textContent?.trim();
                    const rowDate = new Date(rowDateText);
                    if (rowDate < from || rowDate > to) show = false;
                }

                row.style.display = show ? '' : 'none';
            });
        }
        document.addEventListener('DOMContentLoaded', function () {
            var tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'));
            tooltipTriggerList.forEach(function (tooltipTriggerEl) {
                new bootstrap.Tooltip(tooltipTriggerEl);
            });
        });
        let recommendationPrefsExist = false; // simulate backend check

        // Handle form submission (only frontend)
        document.getElementById('recommendationForm')?.addEventListener('submit', function (e) {
            e.preventDefault();
            // Normally this data would be sent to the server
            const formData = Object.fromEntries(new FormData(this));
            console.log('Preferences Saved:', formData);
            bootstrap.Modal.getInstance(document.getElementById('recommendationModal')).hide();
        });

    </script>
    <script src="https://cdn.sheetjs.com/xlsx-0.19.3/package/dist/xlsx.full.min.js"></script>


    <style>
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

        #resetPrefsBtn {
            height: 30px;
            width: 30px;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
        }

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
    </style>
@endsection