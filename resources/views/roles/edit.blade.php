@extends('layout.app')
@section('content')
   <div class="col-xl-12 box-col-6 p-3">
    <div class="row">
        <div class="col-md-9">
            <div class="card">
                <div class="card-body">
                    <h5 class="mb-3">Role Edit</h5>
                    <form class="card-body" method="POST" action="{{ route('roles.update', $role->id) }}">
                        @csrf
                        @method('PUT')
                        <div class="row g-4">
                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <input type="text" id="multicol-username" class="form-control" placeholder="Enter Name"
                                        name="name" value="{{ $role->name }}" />
                                    <label for="multicol-username">Role</label>
                                </div>
                            </div>

                            @php
                                $groupedPermissions = $permission->groupBy(function ($item) {
                                    return explode('-', $item->name)[0];
                                });
                            @endphp

                            @foreach ($groupedPermissions as $group => $permissions)
                                <div class="bg-light-primary rounded-2">
                                    <h6 class="my-2 ms-2 text-black">{{ ucwords(str_replace('_', ' ', $group)) }}</h6>
                                </div>
                                <div class="row row-bordered g-0">
                                    @foreach ($permissions as $value)
                                        <div class="col-md-3 pt-0 p-3">
                                            <div class="form-check mt-3">
                                                <input class="form-check-input" type="checkbox" value="{{ $value->id }}"
                                                    id="permission-{{ $value->id }}" name="permission[{{ $value->id }}]"
                                                    {{ in_array($value->id, $rolePermissions) ? 'checked' : '' }} />
                                                <label class="form-check-label text-capitalize">
                                                    {{ ucwords(str_replace('-', ' ', Str::after($value->name, '-'))) }}
                                                </label>
                                            </div>
                                        </div>
                                    @endforeach
                                </div>
                            @endforeach
                        </div>
                        <div class="d-flex flex-row-reverse gap-1 mt-2">
                            <button type="submit" class="btn btn-outline-primary">Submit</button>
                            <a href="{{ route('roles.index') }}" type="back" class="btn btn-outline-secondary">Back</a>
                        </div>
                    </form>
                </div>
            </div>
        </div>
        <div class="col-md-3">
            <div class="card bg-secondary text-white text-center d-flex align-items-center justify-content-center" style="height: 380px; border: 2px solid var(--bs-light); background-color: rgba(255,255,255,0.1);">
                <div id="user-card" class="card-body position-relative p-4 d-flex flex-column justify-content-center align-items-center">
                
                <!-- Watermark Icon -->
                <i id="card-icon" data-feather="" style="position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); width:170px; height:170px; opacity:0.1; pointer-events:none; z-index:0;"></i>
                
                <!-- Foreground Content Box -->
                <div style="z-index:1;">
                    <h5 id="card-title" class="fw-bold mb-2 text-uppercase"></h5>
                    <p class="mt-2" id="card-desc" style="font-size: 14px; line-height: 1.5; margin-bottom: 0;"></p>
                </div>

                </div>
            </div>
        </div>


    </div> <!-- End of .row -->
</div>
<script>
  document.addEventListener("DOMContentLoaded", function () {
    const userType = @json($role->name).toLowerCase();

    const userData = {
      carrier: {
        title: "Carrier",
        icon: "truck",
        desc: "Smart load matching and instant alerts based on equipment, routes, and certifications (e.g., HAZMAT, TWIC). Includes vehicle/cargo type filters, real-time rate tools, load chaining, and compliance checks. AI suggests loads using driver and route data, while document upload and verification tools ensure eligibility. The system also notifies carriers about maintenance deadlines and provides analytics to improve delivery performance."
      },
      shipper: {
        title: "Shipper",
        icon: "globe",
        desc: "Central dashboard for full shipment tracking and management, with AI-based transport mode selection. Offers digital document handling, load creation tools, real-time ETAs, and carrier vetting. Enables cargo specification input, credit tracking, invoicing, and carbon footprint estimation. The dashboard also helps shippers forecast demand and improve supplier coordination using machine learning predictions."
      },
      "freight forwarder": {
        title: "Freight Forwarder",
        icon: "package",
        desc: "Live pricing and capacity data across all modes, automated contracts, and transparent load tracking. Offers white-label portals, onboarding tools, custom markups, and load board management. Features include negotiation chat, backhaul intelligence, and integrated billing and payments. Forwarders can also automate customer updates and create reports for performance analysis and compliance."
      },
      broker: {
        title: "Broker",
        icon: "shuffle",
        desc: "Enables hybrid shipping with flexible, multi-leg routing using all transport modes. Built-in routing AI considers port delays and costs to suggest best paths for global and local deliveries. Brokers can also analyze market trends, track margins, and ensure compliance with regulatory documentation using built-in tools."
      },
      admin: {
        title: "Admin",
        icon: "settings",
        desc: "Full system control for managing users, permissions, dashboards, and master data across all modules and roles. Provides real-time analytics, incident logs, and audit trails to maintain system integrity and role-based access. Admins also monitor API health, control feature rollouts, and assign dynamic roles using smart filters and workflow automation."
      }
    };

    const user = userData[userType];

    if (user) {
      const titleEl = document.getElementById("card-title");
      const descEl = document.getElementById("card-desc");
      const iconEl = document.getElementById("card-icon");

      if (titleEl) titleEl.textContent = user.title;
      if (descEl) descEl.textContent = user.desc;
      if (iconEl) iconEl.setAttribute("data-feather", user.icon);

      feather.replace();
    } else {
      console.warn("Unknown or missing user type:", userType);
    }
  });
</script>



@endsection
