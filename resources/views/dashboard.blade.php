@extends('layout.app')
@section('content')
    <div>
        <div>
            <div class="page-title">
                <div class="row">
                    <div class="col-6">
                        <h4>Dashboard</h4>
                    </div>
                    <div class="col-6">
                        <ol class="breadcrumb">
                            <li class="breadcrumb-item"><a href="{{ route('dashboard') }}">
                                    <i data-feather="home"></i>
                                </a></li>
                            <li class="breadcrumb-item">Dashboard</li>
                            <li class="breadcrumb-item active">Overview</li>
                        </ol>
                    </div>
                </div>
            </div>
        </div>

        <!-- Container-fluid starts-->
        <div class="container-fluid">
            <div class="row widget-grid">

                <!-- Welcome Card -->
                <div class="col-xxl-4 col-sm-6 box-col-6 mb-4">
                    <div class="card profile-box h-100">
                        <div class="card-body">
                            <div class="media media-wrapper justify-content-between">
                                <div class="media-body">
                                    <div class="greeting-user">
                                        <h4 class="f-w-600">Welcome Back, {{ Auth::user()->name ?? 'User' }}!</h4>
                                        <p>Here’s what’s happening in your account today</p>
                                    </div>
                                </div>
                                <div>
                                    <div class="clockbox">
                                        <i data-feather="clock" class="text-white" style="width:45px;height:45px;"></i>
                                    </div>
                                    <div class="badge f-10 p-0" id="txt">{{ now()->format('l, d M Y') }}</div>
                                </div>
                            </div>
                            <div class="cartoon">
                                <img class="img-fluid" src="../assets/images/dashboard/cartoon.svg" alt="vector">
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Widgets -->
                <div class="col-12 col-xl-5 mb-4">
                    <div class="row g-2">
                        {{-- WIDGETS FOR CARRIER --}}
                        @if ($role_id === 3)
                            <div class="col-6">
                                <div class="card small-widget h-100">
                                    <div class="card-body primary">
                                        <span class="f-light">Completed Legs</span>
                                        <h5>{{ $completed_legs }}</h5>
                                        <div class="bg-gradient"><i data-feather="check-circle" class="font-primary"></i></div>
                                    </div>
                                </div>
                            </div>
                            <div class="col-6">
                                <div class="card small-widget h-100">
                                    <div class="card-body success">
                                        <span class="f-light">On-Time Rate</span>
                                        <h5>{{ $onTimeRate }}%</h5>
                                        <div class="bg-gradient"><i data-feather="clock" class="font-success"></i></div>
                                    </div>
                                </div>
                            </div>
                            <div class="col-6">
                                <div class="card small-widget h-100">
                                    <div class="card-body warning">
                                        <span class="f-light">Revenue (This Month)</span>
                                        <h5>${{ number_format($carrierRevenueMonth, 2) }}</h5>
                                        <div class="bg-gradient"><i data-feather="dollar-sign" class="font-warning"></i></div>
                                    </div>
                                </div>
                            </div>
                            <div class="col-6">
                                <div class="card small-widget h-100">
                                    <div class="card-body secondary">
                                        <span class="f-light">Payout Time</span>
                                        <h5>{{ $avgPayoutDays }} Days Avg</h5>
                                        <div class="bg-gradient"><i data-feather="credit-card" class="font-secondary"></i></div>
                                    </div>
                                </div>
                            </div>
                        @else
                            {{-- WIDGETS FOR OTHER USERS --}}
                            <div class="col-6">
                                <div class="card small-widget h-100">
                                    <div class="card-body success">
                                        <span class="f-light">On-Time Pickup %</span>
                                        <h5>{{ $onTimePickupRate }}%</h5>
                                        <div class="bg-gradient"><i data-feather="truck" class="font-success"></i></div>
                                    </div>
                                </div>
                            </div>
                            <div class="col-6">
                                <div class="card small-widget h-100">
                                    <div class="card-body warning">
                                        <span class="f-light">On-Time Delivery %</span>
                                        <h5>{{ $onTimeDeliveryRate }}%</h5>
                                        <div class="bg-gradient"><i data-feather="package" class="font-warning"></i></div>
                                    </div>
                                </div>
                            </div>
                            <div class="col-6">
                                <div class="card small-widget h-100">
                                    <div class="card-body secondary">
                                        <span class="f-light">Active Legs</span>
                                        <h5>{{ $activeLegs }}</h5>
                                        <div class="bg-gradient"><i data-feather="activity" class="font-secondary"></i></div>
                                    </div>
                                </div>
                            </div>
                            <div class="col-6">
                                <div class="card small-widget h-100">
                                    <div class="card-body primary">
                                        <span class="f-light">Monthly Spend</span>
                                        <h5>${{ number_format($monthlySpend, 2) }}</h5>
                                        <div class="bg-gradient"><i data-feather="dollar-sign" class="font-primary"></i></div>
                                    </div>
                                </div>
                            </div>
                        @endif
                    </div>
                </div>

                <!-- Pending Requests -->
                <div class="col-12 col-xl-3 mb-4">
                    <div class="card h-100">
                        <div class="card-body d-flex align-items-center">
                            <div class="course-widget d-flex gap-3 align-items-center">
                                <div class="course-icon">
                                    <svg class="fill-icon">
                                        <use href="../assets/svg/icon-sprite.svg#course-1"></use>
                                    </svg>
                                </div>
                                <div>
                                    <h5 class="mb-0">{{ $newRequests }}</h5>
                                    <span class="f-light">New Load Requests</span>

                                    @if($newRequestsToday > 0)
                                        <span class="badge bg-warning text-dark my-2 p-2 rounded-4 d-block">
                                            {{ $newRequestsToday }} new today
                                        </span>
                                    @else
                                        <span class="badge bg-light text-dark my-2 p-2 rounded-4 d-block">
                                            No new today
                                        </span>
                                    @endif
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Recent Activity (Upcoming Schedule) -->
                <div class="col-xxl-4 col-ed-6 col-md-7 box-col-7">
                    <div class="card schedule-card py-2">
                        <div class="card-header card-no-border">
                            <div class="header-top">
                                <h5 class="m-0">Recent Activity</h5>
                            </div>
                        </div>
                        <div class="card-body pt-0" style="max-height: 250px; overflow-y: auto;">
                            <ul class="schedule-list">
                                @forelse ($activities as $a)

                                    @php
                                        // determine icon + color
                                        $icon = 'clock';
                                        $color = 'secondary';
                                        $text = '';
                                        $sub = $a->created_at->format('M d • h:i A');

                                        if ($a->activity_type === 'event') {
                                            $icon = 'truck';
                                            $color = 'primary';
                                            $text = "Leg #{$a->leg_id} " . ucwords(str_replace('_', ' ', $a->type));
                                        }

                                        if ($a->activity_type === 'document') {
                                            $icon = 'file-text';
                                            $color = 'warning';
                                            $text = "Document Uploaded: " . strtoupper($a->type);
                                        }

                                        if ($a->activity_type === 'status') {
                                            $icon = 'check-circle';
                                            $color = 'success';
                                            $text = "{$a->type} (Leg #{$a->leg_id})";
                                        }
                                    @endphp

                                    <li class="{{ $color }}">
                                        <i data-feather="{{ $icon }}" class="me-2 text-{{ $color }}"></i>
                                        <div>
                                            <h6 class="mb-1">{{ $text }}</h6>
                                            <span class="f-light">{{ $sub }}</span>
                                        </div>
                                    </li>

                                @empty
                                    <li><span class="text-muted">No recent activity.</span></li>
                                @endforelse
                            </ul>
                        </div>
                    </div>
                </div>

                <!-- Documents (Active Lessons section) -->
                <div class="col-xxl-3 col-ed-6 col-md-5 col-sm-6 box-col-5">
                    <div class="card py-2">
                        <div class="card-header card-no-border">
                            <div class="header-top">
                                <h5 class="m-0">Your Documents</h5>
                            </div>
                        </div>
                        <div class="card-body pt-0" style="max-height: 250px; overflow-y: auto;">
                            <ul class="lessons-lists">
                                @foreach ($documents as $doc)
                                    <li>
                                        <i data-feather="file-text" class="me-2 text-primary"
                                            style="width:25px;height:25px;"></i>
                                        <div>
                                            <h6 class="f-14 mb-0">{{ $doc->name }}</h6>
                                            <span class="f-light">Uploaded {{ $doc->uploaded_at->diffForHumans() }}</span>
                                        </div>
                                        <span class="ms-auto">
                                            <!-- Open the document in a new window -->
                                            <a href="{{ asset('storage/' . $doc->file_path) }}" target="_blank"
                                                data-bs-toggle="tooltip" data-bs-placement="top" title="View Document">
                                                <i data-feather="eye" class="badge rounded-circle p-2 badge-primary"></i>
                                            </a>
                                        </span>
                                    </li>
                                @endforeach
                            </ul>
                        </div>
                    </div>
                </div>


                <!-- All Campaigns Section -->
                <div class="col-xl-12">
                    <div class="card">
                        <div class="card-header card-no-border">
                            <div class="header-top">
                                <h5 class="m-0">Today's Pickups & Deliveries</h5>
                            </div>
                        </div>
                        <div class="card-body pt-0 campaign-table">
                            <div class="recent-table table-responsive">
                                <table class="table">
                                    <thead>
                                        <tr>
                                            <th>Load ID</th>
                                            <th>Pickup</th>
                                            <th>Delivery</th>
                                            <th>Status</th>
                                            @if ($role_id != 3)
                                                <th>Carrier</th>
                                            @endif
                                        </tr>
                                    </thead>
                                    <tbody>
                                        @foreach ($loadLegs as $leg)
                                            <tr>
                                                <td>{{ $leg->leg_code }}</td>
                                                <td>{{ $leg->pickup_date->format('Y-m-d') }}</td>
                                                <td>{{ $leg->delivery_date->format('Y-m-d') }}</td>
                                                <td>
                                                    <span class="badge 
                                                        @switch($leg->status_master->id)
                                                            @case(10)
                                                                bg-success
                                                                @break
                                                            @case(9)
                                                                bg-info
                                                                @break
                                                            @case(4)
                                                                bg-warning
                                                                @break
                                                            @default
                                                                bg-secondary
                                                        @endswitch
                                                    ">
                                                        {{ $leg->status_master->name }}
                                                    </span>
                                                </td>
                                                @if ($role_id != 3)
                                                    <td>{{ $leg->carrier->name ?? 'N/A' }}</td>
                                                @endif
                                            </tr>
                                        @endforeach
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <!-- Container-fluid Ends-->
        </div>
    </div>
@endsection

@section('script')
    <script>
        feather.replace();
    </script>
@endsection