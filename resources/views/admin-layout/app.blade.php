<!DOCTYPE html>
<html lang="en">

<head>
    <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description"
        content="Cuba admin is super flexible, powerful, clean &amp; modern responsive bootstrap 5 admin template with unlimited possibilities.">
    <meta name="keywords"
        content="admin template, Cuba admin template, dashboard template, flat admin template, responsive admin template, web app">
    <meta name="author" content="pixelstrap">
    <link rel="icon" href="{{ url('assets/images/favicon.png') }}" type="image/x-icon">
    <link rel="shortcut icon" href="{{ url('assets/images/favicon.png') }}" type="image/x-icon">
    <title>ST Loads - Logistic Company</title>
    <!-- Google font-->
    <link href="https://fonts.googleapis.com/css?family=Rubik:400,400i,500,500i,700,700i&amp;display=swap"
        rel="stylesheet">
    <link href="https://fonts.googleapis.com/css?family=Roboto:300,300i,400,400i,500,500i,700,700i,900&amp;display=swap"
        rel="stylesheet">
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/font-awesome.css') }}">
    <!-- ico-font-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/icofont.css') }}">
    <!-- Themify icon-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/themify.css') }}">
    <!-- Flag icon-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/flag-icon.css') }}">
    <!-- Feather icon-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/feather-icon.css') }}">
    <!-- Plugins css start-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/slick.css') }}">
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/slick-theme.css') }}">
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/scrollbar.css') }}">
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/animate.css') }}">
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/datatables.css') }}">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/sweetalert2@11.22.4/dist/sweetalert2.min.css">



    <!-- Plugins css Ends-->
    <!-- Bootstrap css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/bootstrap.css') }}">
    <!-- App css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/style.css') }}">
    <link id="color" rel="stylesheet" href="{{ url('assets/css/color-1.css') }}" media="screen">
    <!-- Responsive css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/custom-responsive.css') }}">
    <!-- Dark Mode css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/dark-mode.css') }}">
    <!-- <link rel="stylesheet" type="text/css" href="{{ url('assets/css/responsive.css') }}"> -->

</head>

<body>
    <!-- Toast Container -->
    @include('components.toast-container')
    
    <div class="main-wrapper d-flex flex-column min-vh-100"
        style="background: url('{{ url('assets/images/login/texture-bg.jpg') }}') no-repeat center center / cover;">

        <!-- Logo and Header -->
        <div class="d-flex justify-content-between align-items-center pt-4 px-4 my-4">
            <div style="width: 40px;"></div> <!-- Spacer for centering -->
            <img src="{{ url('assets/images/logo/logo-white.png') }}" alt="Logo" style="height: 100px;">
            <button class="mode-toggle-btn admin-dark-toggle" id="darkModeToggle" type="button" title="Toggle Dark Mode">
                <i data-feather="moon" class="moon-icon"></i>
                <i data-feather="sun" class="sun-icon d-none"></i>
            </button>
        </div>

        <!-- Main Content -->
        <div class="flex-grow-1 d-flex overflow-hidden mt-3">
            <div class="container-fluid h-100">
                <div class="row h-100 g-0">
                    <!-- Sidebar -->
                    @include('admin-layout.sidebar')

                    <!-- Content Area -->
                    <div class="col overflow-auto px-3">
                        @yield('content')
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Footer -->
    <footer class="footer mt-auto bg-transparent py-2">
        <div>
            <div class="row">
                <div class="col text-center text-secondary">
                    <p class="mb-0">© 2025 Load Board All Rights Reserved</p>
                </div>
            </div>
        </div>
    </footer>


    <!-- latest jquery-->
    <script src="{{ url('assets/js/jquery.min.js') }}"></script>
    <!-- Bootstrap js-->
    <script src="{{ url('assets/js/bootstrap/bootstrap.bundle.min.js') }}"></script>
    <!-- feather icon js-->
    <script src="{{ url('assets/js/icons/feather-icon/feather.min.js') }}"></script>
    <script src="{{ url('assets/js/icons/feather-icon/feather-icon.js') }}"></script>
    <!-- scrollbar js-->
    <script src="{{ url('assets/js/scrollbar/simplebar.js') }}"></script>
    <script src="{{ url('assets/js/scrollbar/custom.js') }}"></script>
    <!-- Sidebar jquery-->
    <script src="{{ url('assets/js/config.js') }}"></script>
    <!-- Plugins JS start-->
    <script src="{{ url('assets/js/sidebar-menu.js') }}"></script>
    <script src="{{ url('assets/js/sidebar-pin.js') }}"></script>
    <script src="{{ url('assets/js/clock.js') }}"></script>
    <script src="{{ url('assets/js/slick/slick.min.js') }}"></script>
    <script src="{{ url('assets/js/slick/slick.js') }}"></script>
    <script src="{{ url('assets/js/header-slick.js') }}"></script>
    <script src="{{ url('assets/js/chart/apex-chart/apex-chart.js') }}"></script>
    <script src="{{ url('assets/js/chart/apex-chart/stock-prices.js') }}"></script>
    <script src="{{ url('assets/js/chart/apex-chart/moment.min.js') }}"></script>
    <script src="{{ url('assets/js/notify/bootstrap-notify.min.js') }}"></script>
    <script src="{{ url('assets/js/dashboard/default.js') }}"></script>
    <script src="{{ url('assets/js/notify/index.js') }}"></script>
    <script src="{{ url('assets/js/typeahead/handlebars.js') }}"></script>
    <script src="{{ url('assets/js/typeahead/typeahead.bundle.js') }}"></script>
    <script src="{{ url('assets/js/typeahead/typeahead.custom.js') }}"></script>
    <script src="{{ url('assets/js/typeahead-search/handlebars.js') }}"></script>
    <script src="{{ url('assets/js/typeahead-search/typeahead-custom.js') }}"></script>
    <script src="{{ url('assets/js/height-equal.js') }}"></script>
    <script src="{{ url('assets/js/animation/wow/wow.min.js') }}"></script>
    <script src="{{ url('assets/js/datatable/datatables/jquery.dataTables.min.js') }}"></script>
    <script src="{{ url('assets/js/datatable/datatables/datatable.custom.js') }}"></script>
    <script src="{{ url('assets/js/dashboard/dashboard_4.js') }}"></script>
    <script src="{{ url('assets/js\general-widget.js') }}"></script>
    <script src="{{ url('assets/js/animation/wow/wow.min.js') }}"></script>
    <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11.22.4/dist/sweetalert2.all.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/apexcharts"></script>


    <!-- Plugins JS Ends-->
    <!-- Theme js-->
    <script src="{{ url('assets/js/script.js') }}"></script>
    <script src="{{ url('assets/js/theme-customizer/customizer.js') }}"></script>
    <script>
        new WOW().init();
    </script>

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
    
    <!-- Dark Mode Toggle Script -->
    <script>
        document.addEventListener('DOMContentLoaded', function() {
            const darkModeToggle = document.getElementById('darkModeToggle');
            const body = document.body;
            const moonIcon = document.querySelector('.moon-icon');
            const sunIcon = document.querySelector('.sun-icon');
            
            // Check for saved dark mode preference
            const isDarkMode = localStorage.getItem('darkMode') === 'enabled';
            
            // Apply saved preference on load
            if (isDarkMode) {
                body.classList.add('dark-mode');
                if (moonIcon) moonIcon.classList.add('d-none');
                if (sunIcon) sunIcon.classList.remove('d-none');
            }
            
            // Toggle dark mode
            if (darkModeToggle) {
                darkModeToggle.addEventListener('click', function() {
                    body.classList.toggle('dark-mode');
                    
                    if (body.classList.contains('dark-mode')) {
                        localStorage.setItem('darkMode', 'enabled');
                        if (moonIcon) moonIcon.classList.add('d-none');
                        if (sunIcon) sunIcon.classList.remove('d-none');
                        
                        // Show toast notification
                        if (typeof toastInfo === 'function') {
                            toastInfo('Dark mode enabled', 'Theme');
                        }
                    } else {
                        localStorage.setItem('darkMode', 'disabled');
                        if (moonIcon) moonIcon.classList.remove('d-none');
                        if (sunIcon) sunIcon.classList.add('d-none');
                        
                        // Show toast notification
                        if (typeof toastInfo === 'function') {
                            toastInfo('Light mode enabled', 'Theme');
                        }
                    }
                    
                    // Re-initialize feather icons after toggle
                    if (typeof feather !== 'undefined') {
                        feather.replace();
                    }
                });
            }
        });
    </script>
    
    @stack('scripts')

</body>

</html>
