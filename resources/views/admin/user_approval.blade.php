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
                    <div class="d-flex justify-content-between align-items-center flex-wrap">
                        <div>
                            <h4>User Approval List</h4>
                            <span>Approve or reject users below.</span>
                        </div>
                        <div class="mb-2" style="min-width: 300px;">
                            <input type="text" id="searchUserApproval" class="form-control form-control-sm" placeholder="Search by Name, Email, Role...">
                        </div>
                    </div>
                </div>
                <div class="card-body">
                    <div class="table-responsive user-datatable">
                        <div style="height: 800px; overflow: auto;">
                            <table class="table table-striped align-middle text-nowrap" id="user-approval-table">
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
                                            <td class="d-flex gap-1">
                                                <a href="{{ route('user.profile', $user->id) }}"
                                                    class="btn btn-info btn-sm w-80">Profile</a>
                                                @if ($user->stripe_connect_account_id && $user->roles()->first()->id == 3)
                                                    <button type="button" data-bs-toggle="modal"
                                                        data-bs-target="#updateStatus-{{ $user->id }}"
                                                        class="btn btn-primary btn-sm w-80">Action</button>
                                                @elseif($user->roles()->first()->id == 3)
                                                    <p>Payout configuration needed</p>
                                                @else
                                                    <button type="button" data-bs-toggle="modal"
                                                        data-bs-target="#updateStatus-{{ $user->id }}"
                                                        class="btn btn-primary btn-sm w-80">Action</button>
                                                @endif
                                            </td>
                                        </tr>
                                    @empty
                                        <tr>
                                            <td colspan="6" class="text-center">No data available.</td>
                                        </tr>
                                    @endforelse
                                </tbody>
                            </table>
                            @foreach ($users as $user)
                                <div class="modal fade" id="updateStatus-{{ $user->id }}" tabindex="-1" aria-hidden="true">
                                    <div class="modal-dialog modal-md modal-dialog-centered">
                                        <div class="modal-content border-0 shadow-sm rounded-3">
                                            <div class="modal-header border-0">
                                                <h5 class="modal-title">User Forwarding</h5>
                                                <button type="button" class="btn-close" data-bs-dismiss="modal"
                                                    aria-label="Close"></button>
                                            </div>
                                            <form method="POST" action="{{ route('user.update-status', $user->id) }}">
                                                @csrf
                                                <div class="modal-body">
                                                    <div class="mb-3">
                                                        <label for="remarks" class="form-label fw-medium">Remarks</label>
                                                        <textarea class="form-control" name="remarks" id="remarks" rows="3"
                                                            placeholder="Enter remarks (optional for Approve, required for Reject/Send Back)"></textarea>
                                                    </div>
                                                </div>
                                                <div class="modal-footer border-0 d-flex justify-content-end gap-1">
                                                    <button type="submit" class="btn btn-secondary btn-sm" name="status"
                                                        value="5">Send Back</button>
                                                    <button type="submit" class="btn btn-primary btn-sm" name="status"
                                                        value="1">Approve</button>
                                                    <button type="submit" class="btn btn-danger btn-sm" name="status"
                                                        value="2">Reject</button>
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

<script>
    document.addEventListener('DOMContentLoaded', function() {
        // Search functionality for User Approval table
        const searchInput = document.getElementById('searchUserApproval');
        const table = document.getElementById('user-approval-table');
        
        if (searchInput && table) {
            searchInput.addEventListener('keyup', function() {
                const filter = this.value.toLowerCase();
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
            });
        }
    });
</script>
@endsection
