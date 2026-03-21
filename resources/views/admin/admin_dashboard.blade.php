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

        <!-- ====================== NEW SECTION ADDED HERE ======================= -->
        <div class="col-12 mb-0">
            <div class="row g-3 align-items-stretch">
                <!-- Left Column (3 Radial Cards) -->
                <div class="col-xl-4 col-md-6">
                    <!-- Funds on Hold -->
                    <div class="card widget-hover mb-3">
                        <div class="card-body radial-progress-card">
                            <div>
                                <h6 class="mb-0">Funds on Hold</h6>
                                <div class="sale-details">
                                    <h5 class="font-primary mb-0">${{ number_format($fundsOnHold, 2) }}</h5>
                                </div>
                                <p class="f-light">Pending financial clearance</p>
                            </div>
                        </div>
                    </div>

                    <!-- Payouts Pending -->
                    <div class="card widget-hover mb-3">
                        <div class="card-body radial-progress-card">
                            <div>
                                <h6 class="mb-0">Payouts Pending</h6>
                                <div class="sale-details">
                                    <h5 class="font-secondary mb-0">${{ number_format($fundsOnHold, 2) }}</h5>
                                </div>
                                <p class="f-light">Awaiting finance approval</p>
                            </div>
                        </div>
                    </div>

                    <!-- Approval SLAs -->
                    <div class="card widget-hover">
                        <div class="card-body radial-progress-card">
                            <div>
                                <h6 class="mb-0">Approval SLAs</h6>
                                <div class="sale-details">
                                    <h5 class="font-success mb-0">{{ $approvalSLAPercentage }}%</h5>
                                </div>
                                <p class="f-light">Requests approved within SLA window</p>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Right Column (Recent Activity) -->
                <div class="col-xl-8 col-md-6">
                    <div class="card schedule-card py-2">
                        <div class="card-header card-no-border">
                            <div class="header-top">
                                <h5 class="m-0">Recent Activity</h5>
                            </div>
                        </div>
                        <div class="card-body pt-0" style="max-height: 330px; overflow-y: auto;">
                            <ul class="schedule-list">
                                @foreach ($recentActivities as $activity)
                                    <li class="{{ $activity->color }}">
                                        <i data-feather="{{ $activity->icon }}" class="me-2 text-{{ $activity->color }}"></i>
                                        <div>
                                            <h6 class="mb-1">{{ $activity->message }}</h6>
                                            <span class="f-light">{{ $activity->time }}</span>
                                        </div>
                                    </li>
                                @endforeach
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <!-- ====================== NEW SECTION END ======================= -->


    </div>
@endsection