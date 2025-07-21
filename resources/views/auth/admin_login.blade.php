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
    <style>
        :root {
            --primary-blue: #1F537B;
            --light-blue: #00ADF0;
            --accent-blue: #00ADF0;
        }

        body {
            font-family: 'Poppins', sans-serif;
            background-image: url('../assets/images/login/texture-bg.jpg');
            background-size: cover;
            background-position: center;
            min-height: 100vh;
        }

        .main-container {
            max-width: 1400px;
            width: 95%;
            margin: 3rem auto;
            /* Adds vertical breathing space */
            padding: 2rem 1rem;
            /* Adds inner spacing */
        }

        .welcome-card {
            border-radius: 16px;
            background: rgba(255, 255, 255, 0.97);
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
            border: none;
            overflow: hidden;
            padding: 4rem 2rem !important;
            /* More inner padding */
            min-height: 600px;
            /* Increase height of white card */
        }

        @media (min-height: 700px) {
            .container-fluid {
                min-height: 100vh;
                padding-top: 2rem;
                padding-bottom: 2rem;
            }
        }

        /* Top Logo and Navbar Styling */
        .logo-img-sm {
            max-width: 130px;
            height: auto;
            margin-bottom: 0;
        }

        .navbar {
            padding: 0;
        }

        .navbar-nav {
            flex-direction: row;
        }

        .navbar-nav .nav-link {
            color: #000;
            font-weight: 500;
            padding: 0.5rem 1rem;
            position: relative;
            transition: color 0.3s;
        }

        .navbar-nav .nav-link:hover,
        .navbar-nav .nav-link.active {
            color: var(--primary-blue);
        }

        .navbar-nav .nav-link.active::after {
            content: '';
            display: block;
            height: 3px;
            width: 100%;
            background-color: var(--primary-blue);
            position: absolute;
            bottom: 0;
            left: 0;
        }

        .welcome-title {
            font-size: 1.5rem;
            font-weight: 600;
            color: #111827;
            margin-bottom: 0.5rem;
        }

        .welcome-subtitle {
            font-size: 1.1rem;
            color: #374151;
            font-weight: 500;
            margin-bottom: 0.5rem;
        }

        .welcome-description {
            color: #6b7280;
            font-size: 0.95rem;
            margin-bottom: 2.5rem;
        }

        .role-card {
            border: none;
            border-radius: 12px;
            transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
            background: white;
            box-shadow: 0 6px 15px rgba(0, 0, 0, 0.08);
            height: 100%;
            overflow: hidden;
            position: relative;
            border: 1px solid rgba(0, 0, 0, 0.05);
        }

        .role-card:hover {
            transform: translateY(-8px);
            box-shadow: 0 15px 30px rgba(0, 0, 0, 0.15);
            border-color: var(--light-blue);
        }

        .role-card::after {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 4px;
            background: linear-gradient(90deg, var(--primary-blue), var(--accent-blue));
            transform: scaleX(0);
            transform-origin: left;
            transition: transform 0.4s ease;
        }

        .role-card:hover::after {
            transform: scaleX(1);
        }

        .role-content {
            padding: 2rem 1.5rem;
            text-align: center;
            position: relative;
        }

        .role-icon {
            font-size: 2.8rem;
            margin-bottom: 1.5rem;
            background: linear-gradient(135deg, var(--primary-blue), var(--accent-blue));
            -webkit-background-clip: text;
            background-clip: text;
            color: transparent;
            transition: all 0.3s;
        }

        .role-card:hover .role-icon {
            transform: scale(1.1);
        }

        .role-title {
            font-size: 1.3rem;
            font-weight: 600;
            color: #1f2937;
            margin-bottom: 0.5rem;
        }

        .role-count {
            font-size: 0.9rem;
            color: #6b7280;
            margin-top: 0.5rem;
        }

        .role-arrow {
            position: absolute;
            bottom: 1rem;
            right: 1.5rem;
            color: var(--primary-blue);
            opacity: 0;
            transform: translateX(-10px);
            transition: all 0.3s;
        }

        .role-card:hover .role-arrow {
            opacity: 1;
            transform: translateX(0);
        }
    </style>
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
                                <a class="nav-link" href="{{ route('role') }}">Home</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link active" href="#">Admin</a>
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

                <!-- Role Cards -->
                <div class="row g-4">
                    <div class="login-card login-dark">
                        <div>
                            <div class="login-main">
                                <form class="theme-form" action="{{ route('admin.login.post') }}" method="post">
                                    @csrf
                                    <h4>Sign in to account</h4>
                                    <p>Enter your email & password to login</p>
                                    <div class="row align-items-end">
                                        {{-- Email --}}
                                        <div class="col-md-5 mb-3 mb-md-0">
                                            <label class="col-form-label">Email Address</label>
                                            <div class="input-group">
                                                <input id="email" class="form-control pe-5 rounded-2" type="email"
                                                    name="email" required placeholder="Test@gmail.com">
                                                <input type="hidden" name="id" value="{{ $id ?? '' }}">
                                                <i id="email-icon"
                                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                                            </div>
                                        </div>
                                        {{-- Password --}}
                                        <div class="col-md-5 mb-3 mb-md-0">
                                            <label class="col-form-label">Password</label>
                                            <div class="form-input position-relative">
                                                <input id="password" class="form-control pe-5" type="password"
                                                    name="password" required placeholder="*********">
                                            </div>
                                        </div>
                                        <div class="col-md-2">
                                            <button class="btn btn-primary btn-sm w-100" type="submit">Sign in</button>
                                        </div>
                                    </div>
                                </form>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Bootstrap JS -->
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11"></script>
    @if (session('status'))
        <script>
            Swal.fire({
                toast: true,
                position: 'top-end',
                icon: 'success',
                title: {!! json_encode(session('status')) !!},
                showConfirmButton: false,
                timer: 2500
            }).then(() => {
                @php
                    session(['success' => null]);
                @endphp
            });
        </script>
    @endif

    @if ($errors->any())
        <script>
            Swal.fire({
                toast: true,
                position: 'top-end',
                icon: 'error',
                title: {!! json_encode($errors->first()) !!},
                showConfirmButton: false,
                timer: 2500
            });
        </script>
    @endif

    @if (session('error'))
        <script>
            Swal.fire({
                position: 'center',
                icon: 'error',
                title: {!! json_encode(session('error')) !!},
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
