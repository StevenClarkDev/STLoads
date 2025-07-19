@extends('admin.app')
@section('content')
    <div class="container-fluid p-0 m-0 min-vh-100 d-flex">
        <div class="row g-0 flex-grow-1 mt-3 w-100">
            <!-- Sidebar Column -->
            <div class="col-xl-3 box-col-6 ps-3 pt-3">
                <div class="md-sidebar">
                    <div class="md-sidebar-aside job-left-aside custom-scrollbar">
                        <div class="file-sidebar">
                            <div class="card">
                                <div class="card-body">
                                    <ul>
                                        <li>
                                            <a href="{{ route('user_approval') }}">
                                                <div class="btn btn-primary"><i data-feather="home"></i>Home</div>
                                            </a>
                                        </li>
                                        <li>
                                            <a href="{{ route('carriers') }}">
                                                <div class="btn btn-light"><i data-feather="truck"></i>Carriers</div>
                                            </a>
                                        </li>
                                        <li>
                                            <a href="{{ route('shippers') }}">
                                                <div class="btn btn-light"><i data-feather="box"></i>Shippers</div>
                                            </a>
                                        </li>
                                        <li>
                                            <a href="{{ route('brookers') }}">
                                                <div class="btn btn-light"><i data-feather="user-check"></i>Brookers</div>
                                            </a>
                                        </li>
                                        <li>
                                            <a href="{{ route('freight_forwarders') }}">
                                                <div class="btn btn-light"><i data-feather="send"></i>Freight Forwarders
                                                </div>
                                            </a>
                                        </li>
                                    </ul>
                                    <hr>
                                    <ul>
                                        <li>
                                            <div class="btn btn-outline-primary"><i data-feather="database"></i>Storage
                                            </div>
                                            <div class="m-t-15">
                                                <div class="progress sm-progress-bar mb-1">
                                                    <div class="progress-bar bg-primary" role="progressbar"
                                                        style="width: 25%" aria-valuenow="25" aria-valuemin="0"
                                                        aria-valuemax="100"></div>
                                                </div>
                                                <p>25 GB of 100 GB used</p>
                                            </div>
                                        </li>
                                    </ul>
                                    <hr>
                                    <ul>
                                        <li>
                                            <div class="btn btn-outline-secondary"><i data-feather="log-out"></i>Logout
                                            </div>
                                        </li>
                                    </ul>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <!-- Main Content Column -->
            <div class="col-xl-9 box-col-6 p-3">
                <div class="file-content">
                    <div class="card">
                        <div class="container-fluid">
                            <div class="row mt-3">
                                <div class="col-sm-12">
                                    <div class="row">
                                        <div class="col-xxl-4 col-xl-5 col-md-6 box-col-6">
                                            <div class="card">
                                                <div class="card-header card-no-border pb-0">
                                                    <div class="header-top mb-2">
                                                        <h5>Total Users</h5>
                                                        <div class="dropdown icon-dropdown">
                                                            <button class="btn dropdown-toggle" id="userdropdown"
                                                                type="button" data-bs-toggle="dropdown"
                                                                aria-expanded="false"><i class="icon-more-alt"></i></button>
                                                            <div class="dropdown-menu dropdown-menu-end"
                                                                aria-labelledby="userdropdown">
                                                                <a class="dropdown-item" href="#">Weekly</a>
                                                                <a class="dropdown-item" href="#">Monthly</a>
                                                                <a class="dropdown-item" href="#">Yearly</a>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="card-body py-lg-3 my-2">
                                                    <ul class="user-list">
                                                        <li>
                                                            <div class="user-icon primary">
                                                                <div class="user-box"><i class="font-primary"
                                                                        data-feather="user-plus"></i>
                                                                </div>
                                                            </div>
                                                            <div>
                                                                <h5 class="mb-1">178,098</h5>
                                                                <span class="font-primary d-flex align-items-center">
                                                                    <i class="icon-arrow-up icon-rotate me-1"></i>
                                                                    <span class="f-w-500">+30.89</span>
                                                                </span>
                                                            </div>
                                                        </li>
                                                        <li>
                                                            <div class="user-icon success">
                                                                <div class="user-box"><i class="font-success"
                                                                        data-feather="user-minus"></i>
                                                                </div>
                                                            </div>
                                                            <div>
                                                                <h5 class="mb-1">178,098</h5>
                                                                <span class="font-danger d-flex align-items-center">
                                                                    <i class="icon-arrow-down icon-rotate me-1"></i>
                                                                    <span class="f-w-500">-08.89</span>
                                                                </span>
                                                            </div>
                                                        </li>
                                                    </ul>
                                                </div>
                                            </div>
                                        </div>
                                        <!-- Widget Cards in 2 Rows -->
                                        <div class="col-xxl-5 col-xl-7 col-md-6 box-col-6">
                                            <div class="row gx-2 gy-0">
                                                <!-- Enrolled Carriers -->
                                                <div class="col-md-6">
                                                    <div class="card small-widget mb-2">
                                                        <div class="card-body primary">
                                                            <span class="f-light">Enrolled Carriers</span>
                                                            <div class="d-flex align-items-end gap-1">
                                                                <h4>{{ $carrierCount ?? 15 }}</h4>
                                                                <span class="font-primary f-12 f-w-500">
                                                                    <i class="icon-arrow-up"></i><span>+18%</span>
                                                                </span>
                                                            </div>
                                                            <div class="bg-gradient">
                                                                <i data-feather="truck" class="font-primary"></i>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <!-- Enrolled Brookers -->
                                                <div class="col-md-6">
                                                    <div class="card small-widget mb-2">
                                                        <div class="card-body warning">
                                                            <span class="f-light">Enrolled Brookers</span>
                                                            <div class="d-flex align-items-end gap-1">
                                                                <h4>{{ $brookerCount ?? 12 }}</h4>
                                                                <span class="font-warning f-12 f-w-500">
                                                                    <i class="icon-arrow-up"></i><span>+22%</span>
                                                                </span>
                                                            </div>
                                                            <div class="bg-gradient">
                                                                <i data-feather="user-check" class="font-warning"></i>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <!-- Enrolled Shippers -->
                                                <div class="col-md-6">
                                                    <div class="card small-widget">
                                                        <div class="card-body secondary">
                                                            <span class="f-light">Enrolled Shippers</span>
                                                            <div class="d-flex align-items-end gap-1">
                                                                <h4>{{ $shipperCount ?? 19 }}</h4>
                                                                <span class="font-secondary f-12 f-w-500">
                                                                    <i class="icon-arrow-down"></i><span>-5%</span>
                                                                </span>
                                                            </div>
                                                            <div class="bg-gradient">
                                                                <i data-feather="box" class="font-secondary"></i>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <!-- Enrolled Freight Forwarders -->
                                                <div class="col-md-6">
                                                    <div class="card small-widget">
                                                        <div class="card-body success">
                                                            <span class="f-light">Freight Forwarders</span>
                                                            <div class="d-flex align-items-end gap-1">
                                                                <h4>{{ $freightForwarderCount ?? 21 }}</h4>
                                                                <span class="font-success f-12 f-w-500">
                                                                    <i class="icon-arrow-up"></i><span>+11%</span>
                                                                </span>
                                                            </div>
                                                            <div class="bg-gradient">
                                                                <i data-feather="send" class="font-success"></i>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="col-xxl-3 col-xl-4 col-md-6 box-col-6">
                                            <div class="row">
                                                <div class="col-sm-12">
                                                    <div class="card course-box widget-course">
                                                        <div class="card-body my-3">
                                                            <div class="course-widget">
                                                                <div class="course-icon">
                                                                    <svg class="fill-icon">
                                                                        <use href="../assets/svg/icon-sprite.svg#course-1">
                                                                        </use>
                                                                    </svg>
                                                                </div>
                                                                <div>
                                                                    <h4 class="mb-0">10+</h4>
                                                                    <span class="f-light">Pending Requests</span>
                                                                    <span
                                                                        class="badge bg-secondary text-white my-3 p-2 rounded-4">
                                                                        5 new requests today
                                                                    </span>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        <ul class="square-group">
                                                            <li class="square-1 warning"></li>
                                                            <li class="square-1 primary"></li>
                                                            <li class="square-2 warning1"></li>
                                                            <li class="square-3 danger"></li>
                                                            <li class="square-4 light"></li>
                                                            <li class="square-5 warning"></li>
                                                            <li class="square-6 success"></li>
                                                            <li class="square-7 success"></li>
                                                        </ul>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-sm-12">
                                    <div class="card">
                                        <div class="card-header pb-0 card-no-border">
                                            <h4>User Approval List</h4>
                                            <span>Approve or reject users below.</span>
                                        </div>
                                        <div class="card-body">
                                            <div class="table-responsive user-datatable">
                                                <table class="table table-striped" id="user-approval-table">
                                                    <thead>
                                                        <tr>
                                                            <th>S No.</th>
                                                            <th>Name</th>
                                                            <th>Email</th>
                                                            <th>Role</th>
                                                            <th>Date</th>
                                                            <th>Action</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        @foreach($users as $user)
                                                            <tr>
                                                                <td>{{ $user->id }}</td>
                                                                <td>
                                                                    @if($user->avatar)
                                                                        <img class="img-fluid table-avtar"
                                                                            src="{{ asset('storage/' . $user->avatar) }}" alt=""
                                                                            style="width:32px;height:32px;border-radius:50%;margin-right:8px;">
                                                                    @endif
                                                                    {{ $user->name }}
                                                                </td>
                                                                <td>{{ $user->email }}</td>
                                                                <td>User Role</td>
                                                                <td>7th July 2025</td>
                                                                <td>
                                                                    <a href="{{ route('user.profile', $user->id) }}"
                                                                        class="btn btn-info btn-sm">Profile</a>
                                                                    <button type="button" class="btn btn-success btn-sm"
                                                                        data-bs-toggle="modal" data-bs-target="#approveModal" {{ $user->status == 'approved' ? 'disabled' : '' }}>Approve</button>
                                                                    <button type="button" class="btn btn-danger btn-sm"
                                                                        data-bs-toggle="modal" data-bs-target="#rejectModal" {{ $user->status == 'rejected' ? 'disabled' : '' }}>Reject</button>
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
    </div>
    <!-- Approve Modal -->
    <div class="modal fade" id="approveModal" tabindex="-1" aria-labelledby="approveModalLabel" aria-hidden="true"
        style="z-index: 99999;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content border border-primary">
                <div class="modal-header">
                    <h5 class="modal-title">Approval Confirmation</h5>
                    <button type="button" class="btn-close btn-close-white" data-bs-dismiss="modal"
                        aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <p>Are you sure you want to approve this user?</p>
                    <ul class="list-group list-group-flush mb-3">
                        <li class="list-group-item"><strong>User ID:</strong> <span
                                class="user-id">{{ $user->id ?? '' }}</span></li>
                        <li class="list-group-item"><strong>Email:</strong> <span
                                class="user-email">{{ $user->email ?? '' }}</span></li>
                        <li class="list-group-item"><strong>Role:</strong> <span
                                class="user-role">{{ $user->role ?? '' }}</span></li>
                        <li class="list-group-item"><strong>Date:</strong> <span
                                class="current-date">{{ now()->format('Y-m-d') }}</span></li>
                    </ul>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="approveUser({{ $user->id }})">Confirm</button>
                    <button type="button" class="btn btn-dark" data-bs-dismiss="modal">Cancel</button>
                </div>
            </div>
        </div>
    </div>
    <!-- Reject Modal -->
    <div class="modal fade" id="rejectModal" tabindex="-1" aria-labelledby="rejectModalLabel" aria-hidden="true"
        style="z-index: 99999;">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content border border-primary">
                <div class="modal-header">
                    <h5 class="modal-title">Rejection Confirmation</h5>
                    <button type="button" class="btn-close btn-close-white" data-bs-dismiss="modal"
                        aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <p>Are you sure you want to reject this user?</p>
                    <ul class="list-group list-group-flush mb-3">
                        <li class="list-group-item"><strong>User ID:</strong> <span
                                class="user-id">{{ $user->id ?? '' }}</span></li>
                        <li class="list-group-item"><strong>Email:</strong> <span
                                class="user-email">{{ $user->email ?? '' }}</span></li>
                    </ul>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-danger" onclick="rejectUser({{ $user->id }})">Confirm</button>
                    <button type="button" class="btn btn-dark" data-bs-dismiss="modal">Cancel</button>
                </div>
            </div>
        </div>
    </div>

    @push('styles')
         <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/all.min.css">
    @endpush
    <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11"></script>

    <script>
        function approveUser(userId) {
            // Close modal
            $('#approveModal').modal('hide');

            // Simulate success with SweetAlert
            setTimeout(() => {
                Swal.fire({
                    icon: 'success',
                    title: 'User Approved',
                    text: 'The user has been successfully approved!',
                    timer: 2000,
                    showConfirmButton: false
                });

                // TODO: make actual approval request here (via form or AJAX)
            }, 500);
        }

        function rejectUser(userId) {
            $('#rejectModal').modal('hide');

            setTimeout(() => {
                Swal.fire({
                    icon: 'warning',
                    title: 'User Rejected',
                    text: 'The user has been rejected.',
                    timer: 2000,
                    showConfirmButton: false
                });

                // TODO: make actual rejection request here (via form or AJAX)
            }, 500);
        }
    </script>
@endsection
