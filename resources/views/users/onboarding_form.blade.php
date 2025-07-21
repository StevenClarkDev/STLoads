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
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/responsive.css') }}">
    <!-- Latest Font Awesome CDN -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css"
        integrity="sha512-ud4Xw6Z7YP0fH49YHefBGurHkD5xO2YrYug0St+e3QLCUvBLvNJS2E0RzRPRbJcqfZtCvCoz6rSPc6goEASn3w=="
        crossorigin="anonymous" referrerpolicy="no-referrer" />

</head>

<body>
    <div class="container-fluid p-0"
        style="background-image: url('{{ url('assets/images/login/texture-bg.jpg') }}'); background-size: cover; background-position: center; min-height: 100vh;">

        <!-- Logo Section -->
        <div class="d-flex justify-content-center align-items-start pt-4" style="height: 100px;">
            <img src="{{ url('assets/images/logo/logo-white.png') }}" alt="Logo" style="height: 100px;">
        </div>

        <!-- Content Section -->
        <div class="d-flex align-items-start justify-content-center mt-4" style="min-height: calc(100vh - 100px);">
            <div class="container-fluid p-0 m-0 min-vh-100 d-flex">
                <div class="row g-0 flex-grow-1 mt-3 w-100">
                    <div class="col-xl-12 box-col-6 p-3">
                        <div class="card mx-4">
                            <div class="card-body">
                                <h5 class="mb-3">Onboarding Form</h5>
                                <form class="card-body" method="POST"
                                    action="{{ route('onboarding-form-save', $user->id) }}">
                                    @csrf
                                    <div class="row g-4">
                                        <div class="col-md-6">
                                            <div class="form-floating form-floating-outline">
                                                <input type="text" id="multicol-username" class="form-control"
                                                    placeholder="Enter Name" name="name" />
                                                <label for="multicol-username">Company Name</label>
                                            </div>
                                        </div>
                                        <div class="col-md-6">
                                            <div class="form-floating form-floating-outline">
                                                <input type="text" id="multicol-username" class="form-control"
                                                    placeholder="Enter Name" name="name" />
                                                <label for="multicol-username">Company Address</label>
                                            </div>
                                        </div>
                                        @if ($role->id == 2)
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">DOT Number</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">MC Number</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Certificate of Insurance</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Equipment Types</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Driver Roster</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Safety Scorecard</label>
                                                </div>
                                            </div>
                                        @elseif($role->id == 3)
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Business Entity ID (EIN)</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">General Liability Insurance</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Facility Address</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Fulfillment Contact Info</label>
                                                </div>
                                            </div>
                                        @elseif($role->id == 4)
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">FMCSA Broker License No.</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">MC Authority Number</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Bonding Proof Document</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Performance History</label>
                                                </div>
                                            </div>
                                        @elseif($role->id == 5)
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Freight Forwarder License</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Customs License</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Certificate of Insurance</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">International/Intermodal
                                                        Docs</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" id="multicol-username" class="form-control"
                                                        placeholder="Enter Name" name="name" />
                                                    <label for="multicol-username">Port Authority Registration</label>
                                                </div>
                                            </div>
                                        @endif


                                    </div>
                                    <div class="pt-4">
                                        <button type="submit" class="btn btn-primary me-sm-3 me-1">Submit</button>
                                        <a href="{{ route('login', ['id' => $role->id]) }}" type="back"
                                            class="btn btn-outline-secondary">Cancel</a>
                                    </div>
                                </form>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
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
