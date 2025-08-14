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

    <!-- Bootstrap Icons CDN -->
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css">
    <!-- Plugins css Ends-->
    <!-- Bootstrap css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/bootstrap.css') }}">
    <!-- App css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/style.css') }}">
    <link id="color" rel="stylesheet" href="{{ url('assets/css/color-1.css') }}" media="screen">
    <!-- Responsive css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/responsive.css') }}">
</head>

<body onload="startTime()">
    <!-- loader starts-->
    <div class="loader-wrapper">
        <div class="loader-index"> <span></span></div>
        <svg>
            <defs></defs>
            <filter id="goo">
                <fegaussianblur in="SourceGraphic" stddeviation="11" result="blur"></fegaussianblur>
                <fecolormatrix in="blur" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 19 -9" result="goo">
                </fecolormatrix>
            </filter>
        </svg>
    </div>
    <!-- loader ends-->
    <!-- tap on top starts-->
    <div class="tap-top"><i data-feather="chevrons-up"></i></div>
    <!-- tap on tap ends-->
    <!-- page-wrapper Start-->
    <div class="page-wrapper compact-wrapper" id="pageWrapper">
        <!-- Page Header Start-->
        @include('layout.header')
        <!-- Page Header Ends -->
        <!-- Page Body Start-->
        <div class="page-body-wrapper">
            <!-- Page Sidebar Start-->
            @include('layout.sidebar')
            <!-- Page Sidebar Ends-->
            <div class="page-body">
                @yield('content')
            </div>
        </div>
        <!-- Footer -->
        <footer class="footer mt-auto bg-light py-2">
            <div class="container-fluid">
                <div class="row">
                    <div class="col text-center text-secondary">
                        <p class="mb-0">© 2025 Load Board All Rights Reserved</p>
                    </div>
                </div>
            </div>
        </footer>
    </div>
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
    {{-- <script src="{{ url('assets/js/sidebar-menu.js') }}"></script> --}}
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
    <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11.22.4/dist/sweetalert2.all.min.js"></script>
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
</body>

</html>
