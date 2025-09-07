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
                            <span class="f-light">Enrolled Brokers</span>
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
                                        <td>{{ $loop->iteration }}</td>
                                        <td>{{ ucwords($user->name) }}</td>
                                        <td>{{ $user->email }}</td>
                                        <td>
                                            @foreach ($user->getRoleNames() as $v)
                                                {{ $v }}
                                            @endforeach
                                        </td>
                                        <td>{{ $user->created_at->format('jS F Y') }}</td>
                                        <td>
                                            <a href="{{ route('user.profile', $user->id) }}"
                                                class="btn btn-info btn-sm">Profile</a>
                                            <button type="button" data-bs-toggle="modal"
                                                data-bs-target="#updateStatus-{{ $user->id }}"
                                                class="btn btn-primary d-flex align-items-center">
                                                <i class="mdi mdi-cog-outline mdi-20px me-1"></i> Action
                                            </button>
                                        </td>
                                    </tr>
                                @empty
                                    <tr>
                                        <td colspan="6" class="text-center">No data available.</td>
                                    </tr>
                                @endforelse
                            </tbody>
                        </table>
                        @forelse ($users as $user)
                            <div class="modal fade" id="updateStatus-{{ $user->id }}" tabindex="-1"
                                aria-hidden="true">
                                <div class="modal-dialog modal-lg modal-simple modal-enable-otp modal-dialog-centered">
                                    <div class="p-3 modal-content p-md-5">
                                        <div class="py-3 modal-body py-md-0">
                                            <button type="button" class="btn-close" data-bs-dismiss="modal"
                                                aria-label="Close"></button>
                                            <div class="mb-4 text-center">
                                                <h3 class="mb-2">User Forwarding</h3>
                                            </div>
                                            <form method="POST" class="row g-4"
                                                action="{{ route('user.update-status', $user->id) }}">
                                                @csrf
                                                <div class="col-12 col-md-12">
                                                    <div class="form-floating form-floating-outline">
                                                        <textarea class="form-control h-px-100" name="remarks" id="remarks" placeholder="Enter Remarks here..."></textarea>
                                                        <label for="remarks">Remarks</label>
                                                    </div>
                                                </div>
                                                <div class="text-center col-12">
                                                    <button type="submit" class="btn btn-danger me-sm-3 me-1"
                                                        name="status" value="5">
                                                        Send Back
                                                    </button>
                                                    <button type="submit" class="btn btn-primary me-sm-3 me-1"
                                                        name="status" value="1">
                                                        Approved
                                                    </button>
                                                    <button type="submit" class="btn btn-danger me-sm-3 me-1"
                                                        name="status" value="2">
                                                        Reject
                                                    </button>
                                                </div>
                                            </form>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        @empty
                        @endforelse
                    </div>
                </div>
            </div>
        </div>
    </div>
@endsection
