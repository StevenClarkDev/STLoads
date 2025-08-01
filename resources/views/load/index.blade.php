@extends('layouts_old.app')
@section('content')
    <div class="page-title">
        <div class="row">
            <div class="col-6">
                <h4>Manage Loads </h4>
            </div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="index.html">
                            <svg class="stroke-icon">
                                <use href="../assets/svg/icon-sprite.svg#stroke-home"></use>
                            </svg></a></li>
                    <li class="breadcrumb-item">Manage Loads</li>
                    <li class="breadcrumb-item active">All Loads </li>
                </ol>
            </div>
        </div>
    </div>
    <div class="col-12 px-3 py-2">
        <div class="card">
            <div class="card-body p-0">
                <div class="card mx-3">
                    <div class="card-header pb-0 card-no-border">
                        <h4>Loads List</h4>
                        <span>See registered users below.</span>
                    </div>
                    <div class="d-flex align-items-center justify-content-between my-3 mx-4 flex-wrap">
                        <div class="d-flex gap-2 flex-wrap">
                            <button class="btn btn-sm btn-outline-light rounded-4 border active"
                                onclick="switchTab(this, 'all')">All Loads (512)</button>
                            <button class="btn btn-sm btn-outline-light rounded-4 border"
                                onclick="switchTab(this, 'recommended')">Recommended Loads (52)</button>
                            <button class="btn btn-sm btn-outline-light rounded-4 border"
                                onclick="switchTab(this, 'accepted')">Accepted Loads (211)</button>
                            <button class="btn btn-sm btn-outline-light rounded-4 border"
                                onclick="switchTab(this, 'time')">Time-Sensitive (48)</button>
                        </div>
                        <div class="d-flex flex-column align-items-end gap-2 mt-2 mt-md-0">
                            <div class="d-flex gap-2">
                                <button class="btn btn-sm btn-outline-primary px-3">
                                    <i class="bi bi-filter me-1"></i> Filter
                                </button>
                                <button class="btn btn-sm btn-outline-primary px-3">
                                    <i class="bi bi-download me-1"></i> Export
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="card-body">
                        <div class="table-responsive user-datatable">
                            <div style="max-height: 500px; min-height: 210px; overflow-y: auto;">
                                <div class="table-responsive">
                                    <table class="table table-striped align-middle text-nowrap" id="user-approval-table"
                                        style="font-size: 0.875rem;">
                                        <thead class="bg-white" style="position: sticky; top: 0; z-index: 2;">
                                            <tr>
                                                <th>Load ID</th>
                                                <th>Origin</th>
                                                <th>Destination</th>
                                                <th>Schedule</th>
                                                <th>Status</th>
                                                <th>Payment</th>
                                                <th>Lane & Mode</th>
                                                <th>Actions</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <!-- Each row has a tab-type class -->
                                            <tr class="tab-all tab-accepted">
                                                <td>L-2783</td>
                                                <td><a class="badge rounded-circle p-2 badge-primary" href="#"><i
                                                            data-feather="map-pin"></i></a></td>
                                                <td><a class="badge rounded-circle p-2 badge-primary" href="#"><i
                                                            data-feather="map-pin"></i></a></td>
                                                <td>1<sup>st</sup> Aug, 2025</td>
                                                <td><button
                                                        class="btn btn-outline-primary d-flex align-items-center px-1 py-1"
                                                        type="button">Accepted <span
                                                            class="badge rounded-circle badge-light text-secondary ms-2"></span></button>
                                                </td>
                                                <td><span class="badge rounded-pill badge-success p-2">Paid</span></td>
                                                <td><a class="badge p-2 badge-light text-dark" href="#"><i
                                                            data-feather="map-pin"></i></a> SYD - MLB</td>
                                                <td class="d-flex gap-1">
                                                    <a href="#" class="btn btn-info btn-sm w-80">Profile</a>
                                                    <button type="button" class="btn btn-success btn-sm w-80"
                                                        data-bs-toggle="modal"
                                                        data-bs-target="#contactModal">Contact</button>
                                                </td>
                                            </tr>
                                            <tr class="tab-all tab-recommended">
                                                <td>L-2784</td>
                                                <td><a class="badge rounded-circle p-2 badge-primary" href="#"><i
                                                            data-feather="map-pin"></i></a></td>
                                                <td><a class="badge rounded-circle p-2 badge-primary" href="#"><i
                                                            data-feather="map-pin"></i></a></td>
                                                <td>15<sup>th</sup> Jul, 2025</td>
                                                <td><button
                                                        class="btn btn-outline-primary d-flex align-items-center px-1 py-1"
                                                        type="button">Pending <span
                                                            class="badge rounded-circle badge-light text-secondary ms-2"></span></button>
                                                </td>
                                                <td><span class="badge rounded-pill badge-danger p-2">Unpaid</span></td>
                                                <td><a class="badge p-2 badge-light text-dark" href="#"><i
                                                            data-feather="map-pin"></i></a> SYD - MLB</td>
                                                <td class="d-flex gap-1">
                                                    <a href="#" class="btn btn-info btn-sm w-80">Profile</a>
                                                    <button type="button" class="btn btn-success btn-sm w-80"
                                                        data-bs-toggle="modal"
                                                        data-bs-target="#contactModal">Contact</button>
                                                </td>
                                            </tr>
                                            <tr class="tab-all tab-time">
                                                <td>L-2785</td>
                                                <td><a class="badge rounded-circle p-2 badge-primary" href="#"><i
                                                            data-feather="map-pin"></i></a></td>
                                                <td><a class="badge rounded-circle p-2 badge-primary" href="#"><i
                                                            data-feather="map-pin"></i></a></td>
                                                <td>22<sup>nd</sup> Jun, 2025</td>
                                                <td><button
                                                        class="btn btn-outline-primary d-flex align-items-center px-1 py-1"
                                                        type="button">Accepted <span
                                                            class="badge rounded-circle badge-light text-secondary ms-2"></span></button>
                                                </td>
                                                <td><span class="badge rounded-pill badge-success p-2">Paid</span></td>
                                                <td><a class="badge p-2 badge-light text-dark" href="#"><i
                                                            data-feather="map-pin"></i></a> SYD - MLB</td>
                                                <td class="d-flex gap-1">
                                                    <a href="#" class="btn btn-info btn-sm w-80">Profile</a>
                                                    <button type="button" class="btn btn-success btn-sm w-80"
                                                        data-bs-toggle="modal"
                                                        data-bs-target="#contactModal">Contact</button>
                                                </td>
                                            </tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>
                    </div>
                    <!-- end card-body -->
                </div> <!-- inner card -->
            </div> <!-- outer card-body -->
        </div> <!-- outer card -->
    </div>

    <!-- Tab Script -->
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
    </script>

    <style>
        .btn-outline-light.active {
            background-color: #4d6b8a !important;
            color: white !important;
        }
    </style>
@endsection