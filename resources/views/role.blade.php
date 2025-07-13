<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LoadBoard - Where Smart Logistics Begin</title>
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
                                <a class="nav-link active" href="#">Home</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="#">About</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="#">Services</a>
                            </li>
                            <li class="nav-item">
                                <a class="nav-link" href="#">Contact</a>
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
                                <i class="fas fa-arrow-right role-arrow"></i>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6 col-lg-3">
                        <div class="role-card">
                            <div class="role-content">
                                <i class="fas fa-truck-fast role-icon"></i>
                                <h3 class="role-title">Carrier</h3>
                                <p class="role-count">Count 40</p>
                                <i class="fas fa-arrow-right role-arrow"></i>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6 col-lg-3">
                        <div class="role-card">
                            <div class="role-content">
                                <i class="fas fa-handshake-angle role-icon"></i>
                                <h3 class="role-title">Broker</h3>
                                <p class="role-count">Count 40</p>
                                <i class="fas fa-arrow-right role-arrow"></i>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6 col-lg-3">
                        <div class="role-card">
                            <div class="role-content">
                                <i class="fas fa-ship role-icon"></i>
                                <h3 class="role-title">Freight Forwarder</h3>
                                <p class="role-count">Count 40</p>
                                <i class="fas fa-arrow-right role-arrow"></i>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Bootstrap JS -->
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js"></script>
</body>

</html>