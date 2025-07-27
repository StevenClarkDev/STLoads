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

    <!-- Plugins css Ends-->
    <!-- Bootstrap css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/bootstrap.css') }}">
    <!-- App css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/style.css') }}">
    <link id="color" rel="stylesheet" href="{{ url('assets/css/color-1.css') }}" media="screen">
    <!-- Responsive css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/custom-responsive.css') }}">
    <!-- <link rel="stylesheet" type="text/css" href="{{ url('assets/css/responsive.css') }}"> -->
    <!-- Latest Font Awesome CDN -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css"
        integrity="sha512-ud4Xw6Z7YP0fH49YHefBGurHkD5xO2YrYug0St+e3QLCUvBLvNJS2E0RzRPRbJcqfZtCvCoz6rSPc6goEASn3w=="
        crossorigin="anonymous" referrerpolicy="no-referrer" />

</head>

<body>
    <div class="main-wrapper d-flex flex-column min-vh-100"
        style="background: url('{{ url('assets/images/login/texture-bg.jpg') }}') no-repeat center center / cover;">

        <!-- Logo -->
        <div class="d-flex justify-content-center align-items-start pt-4 my-4" style="height: 100px;">
            <img src="{{ url('assets/images/logo/logo-white.png') }}" alt="Logo" style="height: 100px;">
        </div>

        <!-- Main Content -->
        <div class="flex-grow-1 d-flex overflow-hidden mt-3">
            <div class="container-fluid h-100">
                <div class="row h-100 g-0">
                    <!-- Sidebar -->
                    @include('layout.sidebar')

                    <!-- Content Area -->
                    <div class="col overflow-auto px-3">
                        @yield('content')
                    </div>
                </div>
            </div>
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
    <!-- Plugins JS Ends-->
    <!-- Theme js-->
    <script src="{{ url('assets/js/script.js') }}"></script>
    <script src="{{ url('assets/js/theme-customizer/customizer.js') }}"></script>
    <script>
        new WOW().init();
    </script>
</body>

</html>