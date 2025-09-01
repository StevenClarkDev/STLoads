<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="icon" href="{{ url('assets/images/favicon.png') }}" type="image/x-icon">
    <link rel="shortcut icon" href="{{ url('assets/images/favicon.png') }}" type="image/x-icon">
    <title>LoadBoard - Role Select</title>
    <!-- Google Fonts -->
    <link href="https://fonts.googleapis.com/css2?family=Poppins:wght@400;500;600;700&display=swap" rel="stylesheet">
    <!-- Font Awesome -->
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css" rel="stylesheet">
    <!-- Bootstrap CSS -->
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/sweetalert2@11.22.4/dist/sweetalert2.min.css">

</head>

<body>
    <div class="container-fluid p-0 min-vh-100 d-flex align-items-center justify-content-center">
        <div class="main-container">
            <div class="welcome-card p-4 p-md-5 my-4">
                <!-- Top Row with Logo and Menu -->
                <div class="d-flex justify-content-between align-items-start flex-wrap mb-4">
                    <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo"
                        class="logo-img-sm">
                    <nav class="navbar navbar-expand">
                        <ul class="navbar-nav ms-auto">
                            <li class="nav-item">
                                <a class="nav-link active" href="#">Home</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="{{ route('admin.login') }}">Admin</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="https://stloads.com/about-us">About</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="https://stloads.com/services">Services</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="https://stloads.com/contact-us">Contact</a>
                            </li>
                        </ul>
                    </nav>
                </div>

                <!-- Headings -->
                <div class="text-center my-5">
                    <h2 class="welcome-title mb-2">Welcome to LoadBoard – Where Smart Logistics Begin.</h2>
                    <h5 class="welcome-subtitle mt-3">Select your role</h5>
                    <p class="welcome-description">To start your project we need to customize your preferences.</p>
                </div>

                <!-- Role Cards -->
                <div class="row g-4">
                    <div class="col-md-6 col-lg-3">
                        <div class="role-card">
                            <div class="role-content">
                                <i class="fas fa-boxes role-icon"></i>
                                <h3 class="role-title">Shipper</h3>
                                <p class="role-count">Count 40</p>
                                <a href="{{ route('normal-login', ['id' => 2]) }}">
                                    <i class="fas fa-arrow-right role-arrow"></i>
                                </a>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6 col-lg-3">
                        <div class="role-card">
                            <div class="role-content">
                                <i class="fas fa-truck-fast role-icon"></i>
                                <h3 class="role-title">Carrier</h3>
                                <p class="role-count">Count 40</p>
                                <a href="{{ route('normal-login', ['id' => 3]) }}">
                                    <i class="fas fa-arrow-right role-arrow"></i>
                                </a>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6 col-lg-3">
                        <div class="role-card">
                            <div class="role-content">
                                <i class="fas fa-handshake-angle role-icon"></i>
                                <h3 class="role-title">Broker</h3>
                                <p class="role-count">Count 40</p>
                                <a href="{{ route('normal-login', ['id' => 4]) }}">
                                    <i class="fas fa-arrow-right role-arrow"></i>
                                </a>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6 col-lg-3">
                        <div class="role-card">
                            <div class="role-content">
                                <i class="fas fa-ship role-icon"></i>
                                <h3 class="role-title">Freight Forwarder</h3>
                                <p class="role-count">Count 40</p>
                                <a href="{{ route('normal-login', ['id' => 5]) }}">
                                    <i class="fas fa-arrow-right role-arrow"></i>
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script src="{{ url('assets/js/jquery.min.js') }}"></script>
    <!-- Bootstrap js-->
    <script src="{{ url('assets/js/bootstrap/bootstrap.bundle.min.js') }}"></script>
    <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11.22.4/dist/sweetalert2.all.min.js"></script>
 @if (session()->has('success'))
        <script>
            Swal.fire({
                toast: true,
                position: 'top-end',
                icon: 'success',
                title: 'Success',
                text: {!! json_encode(session('success')) !!},
                showConfirmButton: false,
                timer: 2500
            }).then(() => {
                @php
                    session(['success' => null]);
                @endphp
            });
        </script>
    @endif

    @if (session('error'))
        <script>
            Swal.fire({
                position: 'center',
                icon: 'error',
                title: 'Error',
                text: {!! json_encode(session('error')) !!},
                showConfirmButton: false,
                showCloseButton: true,
                allowOutsideClick: false,
                allowEscapeKey: false,
                backdrop: true,
            }).then(() => {
                @php
                    session(['error' => null]);
                @endphp
            });
        </script>
    @endif
</body>

</html>
