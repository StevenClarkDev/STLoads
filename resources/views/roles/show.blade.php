@extends('admin-layout.app')

@section('content')
    <div class="col-xl-12 p-3">
        <div class="card shadow-sm border-0">
            <div class="card-body p-3" style="height: 380px;">
                <div class="d-flex align-items-center mb-3">
                    <div class="me-3 d-flex align-items-center justify-content-center bg-primary rounded-circle"
                        style="width: 50px; height: 50px;">
                        <i id="card-icon" data-feather="" style="width: 30px; height: 30px; color: white;"></i>
                    </div>
                    <div>
                        <h4 class="card-title mb-0" id="card-title">{{ ucfirst($role->name) }}</h4>
                        <small class="text-muted">Role Overview</small>
                    </div>
                </div>

                <hr class="my-3">

                <div class="row mb-3">
                    <div class="col-12 mb-2">
                        <h6 class="text-uppercase mb-1">Name</h6>
                        <p class="mb-0">{{ ucfirst($role->name) }}</p>
                    </div>

                    <div class="col-12">
                        <h6 class="text-uppercase mb-1">Permissions</h6>
                        @if (!empty($rolePermissions))
                            <div class="d-flex flex-wrap gap-2">
                                @foreach ($rolePermissions as $v)
                                    <span class="badge rounded-pill badge-primary">{{ $v->name }}</span>
                                @endforeach
                            </div>
                        @else
                            <p class="text-muted">No permissions assigned.</p>
                        @endif
                    </div>
                </div>

                <hr class="my-3">

                <div>
                    <h6 class="text-uppercase mb-1">Role Details</h6>
                    <p id="card-desc" class="text-muted small mb-0">Loading description...</p>
                </div>

                <div class="d-flex justify-content-end mt-3">
                    <a href="{{ route('roles.index') }}" class="btn btn-outline-secondary">Back</a>
                </div>
            </div>
        </div>
    </div>

    <script>
        document.addEventListener("DOMContentLoaded", function () {
            const userType = @json($role->name).toLowerCase();
            const userData = {
                carrier: {
                    title: "Carrier",
                    icon: "truck",
                    desc: "Smart load matching and alerts based on equipment, routes, and certifications. Includes rate tools, compliance checks, and AI load suggestions."
                },
                shipper: {
                    title: "Shipper",
                    icon: "globe",
                    desc: "AI-based mode selection, shipment tracking, document handling, ETAs, and invoicing with carbon footprint estimation."
                },
                "freight forwarder": {
                    title: "Freight Forwarder",
                    icon: "package",
                    desc: "Live pricing, load tracking, white-label portals, negotiation tools, and billing integrations."
                },
                broker: {
                    title: "Broker",
                    icon: "shuffle",
                    desc: "Flexible multi-leg routing with AI path suggestions based on costs and port delays."
                },
                admin: {
                    title: "Admin",
                    icon: "settings",
                    desc: "Full access to manage users, roles, permissions, and system settings. Oversee platform operations and view analytics dashboards."
                }
            };

            const user = userData[userType];
            if (user) {
                document.getElementById("card-title").textContent = user.title;
                document.getElementById("card-desc").textContent = user.desc;
                document.getElementById("card-icon").setAttribute("data-feather", user.icon);
                feather.replace();
            } else {
                console.warn("Unknown role:", userType);
            }
        });
    </script>
@endsection
