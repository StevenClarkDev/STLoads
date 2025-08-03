@extends('admin-layout.app')
@section('content')
<div class="row gy-3 py-2">
    <!-- Users Summary Card -->
    <div class="col-12 col-xl-4">
        <div class="card h-100">
            <div class="card-header card-no-border">
                <h5>Total Users</h5>
            </div>
            <div class="card-body">
                <ul class="user-list">
                    <li>
                        <div class="user-icon primary">
                            <div class="user-box">
                                <i class="font-primary" data-feather="user-plus"></i>
                            </div>
                        </div>
                        <div>
                            <h5>{{ $totalUsersApproved }}</h5>
                            <span class="font-primary d-flex align-items-center">
                                <i class="icon-arrow-up icon-rotate me-1"></i>
                                <span class="f-w-500">+{{ $totalUsersApprovedThisMonthPercentage }}%</span>
                            </span>
                        </div>
                    </li>
                    <li>
                        <div class="user-icon success">
                            <div class="user-box">
                                <i class="font-success" data-feather="user-minus"></i>
                            </div>
                        </div>
                        <div>
                            <h5>{{ $totalUsersRejected }}</h5>
                            <span class="font-danger d-flex align-items-center">
                                <i class="icon-arrow-down icon-rotate me-1"></i>
                                <span class="f-w-500">-{{ $totalUsersRejectedThisMonthPercentage }}%</span>
                            </span>
                        </div>
                    </li>
                </ul>
            </div>
        </div>
    </div>

    <!-- Widgets -->
    <div class="col-12 col-xl-5">
        <div class="row g-2">
            <div class="col-6">
                <div class="card small-widget h-100">
                    <div class="card-body primary">
                        <span class="f-light">Enrolled Carriers</span>
                        <div class="d-flex align-items-end gap-1">
                            <h4>{{ $totalCarriersApproved ?? 0 }}</h4>
                        </div>
                        <div class="bg-gradient">
                            <i data-feather="truck" class="font-primary"></i>
                        </div>
                    </div>
                </div>
            </div>
            <div class="col-6">
                <div class="card small-widget h-100">
                    <div class="card-body warning">
                        <span class="f-light">Enrolled Brookers</span>
                        <div class="d-flex align-items-end gap-1">
                            <h4>{{ $totalBrookersApproved ?? 0 }}</h4>
                        </div>
                        <div class="bg-gradient">
                            <i data-feather="user-check" class="font-warning"></i>
                        </div>
                    </div>
                </div>
            </div>
            <div class="col-6">
                <div class="card small-widget h-100">
                    <div class="card-body secondary">
                        <span class="f-light">Enrolled Shippers</span>
                        <div class="d-flex align-items-end gap-1">
                            <h4>{{ $totalShipperApproved ?? 0 }}</h4>
                        </div>
                        <div class="bg-gradient">
                            <i data-feather="box" class="font-secondary"></i>
                        </div>
                    </div>
                </div>
            </div>
            <div class="col-6">
                <div class="card small-widget h-100">
                    <div class="card-body success">
                        <span class="f-light">Freight Forwarders</span>
                        <div class="d-flex align-items-end gap-1">
                            <h4>{{ $totalFreightForwardersApproved ?? 0 }}</h4>
                        </div>
                        <div class="bg-gradient">
                            <i data-feather="send" class="font-success"></i>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Pending Requests -->
    <div class="col-12 col-xl-3">
        <div class="card h-100">
            <div class="card-body d-flex align-items-center">
                <div class="course-widget d-flex gap-3 align-items-center">
                    <div class="course-icon">
                        <svg class="fill-icon">
                            <use href="../assets/svg/icon-sprite.svg#course-1"></use>
                        </svg>
                    </div>
                    <div>
                        <h4 class="mb-0">{{ $usersCount }}</h4>
                        <span class="f-light">Pending Requests</span>
                        <span class="badge bg-primary text-white my-2 p-2 rounded-4 d-block">
                            {{ $usersCountToday ?? 0 }} new today
                        </span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- User Approval Table -->
    <div class="col-12">
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
                            @forelse ($users as $user)
                                <tr>
                                    <td>{{ $user->id }}</td>
                                    <td>{{ ucwords($user->name) }}</td>
                                    <td>{{ $user->email }}</td>
                                    <td>
                                        @foreach ($user->getRoleNames() as $v)
                                            {{ $v }}
                                        @endforeach
                                    </td>
                                    <td>{{ $user->created_at->format('jS F Y') }}</td>
                                    <td>
                                        <a href="{{ route('user.profile', $user->id) }}" class="btn btn-info btn-sm">Profile</a>
                                        <button class="btn btn-success btn-sm" data-bs-toggle="modal" data-bs-target="#approveModal">Approve</button>
                                        <button class="btn btn-danger btn-sm" data-bs-toggle="modal" data-bs-target="#rejectModal">Reject</button>
                                    </td>
                                </tr>
                            @empty
                                <tr><td colspan="6" class="text-center">No data available.</td></tr>
                            @endforelse
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    </div>
</div>
@endsection

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
                            class="user-role">{{ isset($user) ? $user->getRoleNames()->implode(', ') : '' }}</span>
                    </li>
                    <li class="list-group-item"><strong>Date:</strong> <span
                            class="current-date">{{ now()->format('Y-m-d') }}</span></li>
                </ul>
            </div>
            <div class="modal-footer">
                <button type="button" class="btn btn-secondary"
                    onclick="approveUser({{ isset($user) ? $user->id : '1' }})">Confirm</button>
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
                    <li class="list-group-item"><strong>Role:</strong> <span
                            class="user-role">{{ isset($user) ? $user->getRoleNames()->implode(', ') : '' }}</span>
                    </li>
                    <li class="list-group-item"><strong>Date:</strong> <span
                            class="current-date">{{ now()->format('Y-m-d') }}</span></li>
                </ul>
            </div>
            <div class="modal-footer">
                <button type="button" class="btn btn-danger"
                    onclick="rejectUser({{ isset($user) ? $user->id : '1' }})">Confirm</button>
                <button type="button" class="btn btn-dark" data-bs-dismiss="modal">Cancel</button>
            </div>
        </div>
    </div>
</div>

<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/all.min.css">
<script src="https://cdn.jsdelivr.net/npm/sweetalert2@11"></script>

<script>
    function approveUser(userId) {
        // Close the modal immediately
        $('#approveModal').modal('hide');

        fetch('/approve-user', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-TOKEN': '{{ csrf_token() }}' // Laravel CSRF token
                },
                body: JSON.stringify({
                    user_id: userId
                })
            })
            .then(response => response.json())
            .then(data => {

                if (data.success) {
                    Swal.fire({
                        icon: 'success',
                        title: 'User Approved',
                        text: data.message || 'The user has been successfully approved!',
                        timer: 2000,
                        showConfirmButton: false
                    });

                    setTimeout(() => {
                        location.reload();
                    }, 2100);

                } else {
                    Swal.fire({
                        icon: 'error',
                        title: 'Approval Failed',
                        text: data.message || 'Something went wrong while approving the user.',
                    });
                }
            })
            .catch(error => {

                Swal.fire({
                    icon: 'error',
                    title: 'Server Error',
                    text: 'An error occurred. Please try again later.',
                });
                console.error('Approval error:', error);
            });
    }

    function rejectUser(userId) {
        $('#rejectModal').modal('hide');

        fetch('/reject-user', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-TOKEN': '{{ csrf_token() }}' // Laravel CSRF token
                },
                body: JSON.stringify({
                    user_id: userId
                })
            })
            .then(response => response.json())
            .then(data => {

                if (data.success) {
                    Swal.fire({
                        icon: 'success',
                        title: 'User Rejected',
                        text: data.message || 'The user has been successfully rejected!',
                        timer: 2000,
                        showConfirmButton: false
                    });

                    setTimeout(() => {
                        location.reload();
                    }, 2100);

                } else {
                    Swal.fire({
                        icon: 'error',
                        title: 'Rejection Failed',
                        text: data.message || 'Something went wrong while rejecting the user.',
                    });
                }
            })
            .catch(error => {

                Swal.fire({
                    icon: 'error',
                    title: 'Server Error',
                    text: 'An error occurred. Please try again later.',
                });
                console.error('Rejecting error:', error);
            });
    }
</script>
