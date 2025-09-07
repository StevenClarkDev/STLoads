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
                                <form class="card-body" method="POST" enctype="multipart/form-data"
                                    action="{{ route('onboarding-form-save', $user->id) }}">
                                    @csrf
                                    <div class="row g-4">
                                        <div class="col-md-6">
                                            <div class="form-floating form-floating-outline">
                                                <input type="text" id="company_name" class="form-control"
                                                    placeholder="Enter Company Name" name="company_name" />
                                                <label for="company_name">Company Name</label>
                                            </div>
                                        </div>
                                        <div class="col-md-6">
                                            <div class="form-floating form-floating-outline">
                                                <input type="text" id="company_address" class="form-control"
                                                    placeholder="Enter Company Address" name="company_address" />
                                                <label for="company_address">Company Address</label>
                                            </div>
                                        </div>
                                        {{-- <div class="col-md-6">
                                            <div class="form-floating form-floating-outline">
                                                <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                    name="cnic_front" class="form-control" />
                                                <label>CNIC Front</label>
                                            </div>
                                        </div>
                                        <div class="col-md-6">
                                            <div class="form-floating form-floating-outline">
                                                <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                    name="cnic_back" class="form-control" />
                                                <label>CNIC Back</label>
                                            </div>
                                        </div> --}}

                                        {{-- Carrier --}}
                                        @if ($role->id == 2)
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="dot_number" class="form-control"
                                                        placeholder="DOT Number" />
                                                    <label>DOT Number</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="mc_number" class="form-control"
                                                        placeholder="MC Number" />
                                                    <label>MC Number</label>
                                                </div>
                                            </div>
                                            {{-- <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="certificate_of_insurance_carrier"
                                                        class="form-control" />
                                                    <label>Certificate of Insurance</label>
                                                </div>
                                            </div> --}}
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="equipment_types" class="form-control"
                                                        placeholder="Equipment Types" />
                                                    <label>Equipment Types</label>
                                                </div>
                                            </div>
                                            {{-- <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="driver_roster" class="form-control" />
                                                    <label>Driver Roster</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="safety_scorecard" class="form-control" />
                                                    <label>Safety Scorecard</label>
                                                </div>
                                            </div> --}}

                                            {{-- Shipper --}}
                                        @elseif($role->id == 3)
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="business_entity_id"
                                                        class="form-control" placeholder="Business Entity ID (EIN)" />
                                                    <label>Business Entity ID (EIN)</label>
                                                </div>
                                            </div>
                                            {{-- <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="general_liability_insurance" class="form-control" />
                                                    <label>General Liability Insurance</label>
                                                </div>
                                            </div> --}}
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="facility_address"
                                                        class="form-control" placeholder="Facility Address" />
                                                    <label>Facility Address</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="fulfillment_contact_info"
                                                        class="form-control" placeholder="Fulfillment Contact Info" />
                                                    <label>Fulfillment Contact Info</label>
                                                </div>
                                            </div>

                                            {{-- Broker --}}
                                        @elseif($role->id == 4)
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="fmcsa_broker_license_no"
                                                        class="form-control" placeholder="FMCSA Broker License No." />
                                                    <label>FMCSA Broker License No.</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="mc_authority_number"
                                                        class="form-control" placeholder="MC Authority Number" />
                                                    <label>MC Authority Number</label>
                                                </div>
                                            </div>
                                            {{-- <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="bonding_proof_document" class="form-control" />
                                                    <label>Bonding Proof Document</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="performance_history" class="form-control" />
                                                    <label>Performance History</label>
                                                </div>
                                            </div> --}}

                                            {{-- Forwarder --}}
                                        @elseif($role->id == 5)
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="freight_forwarder_license"
                                                        class="form-control"
                                                        placeholder="Freight Forwarder License" />
                                                    <label>Freight Forwarder License</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="text" name="customs_license" class="form-control"
                                                        placeholder="Customs License" />
                                                    <label>Customs License</label>
                                                </div>
                                            </div>
                                            {{-- <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="certificate_of_insurance_freight_forwarder"
                                                        class="form-control" />
                                                    <label>Certificate of Insurance</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="international_docs" class="form-control" />
                                                    <label>International/Intermodal Docs</label>
                                                </div>
                                            </div>
                                            <div class="col-md-6">
                                                <div class="form-floating form-floating-outline">
                                                    <input type="file" accept=".jpeg, .jpg, .png, .pdf"
                                                        name="port_authority_registration" class="form-control" />
                                                    <label>Port Authority Registration</label>
                                                </div>
                                            </div> --}}
                                        @endif
                                        <div class="d-flex justify-content-between align-items-center">
                                            <h5 class="card-header">Documents</h5>
                                            <button type="button" class="btn btn-primary h-75"
                                                id="doc-row">Add</button>
                                        </div>
                                        <table class="table table-bordered" id="document-table">
                                            <thead>
                                                <tr>
                                                    <th>#</th>
                                                    <th>Ducument Name</th>
                                                    <th>Ducument Type</th>
                                                    <th>Document</th>
                                                    <th>Action</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                            </tbody>
                                        </table>
                                    </div>

                                    <div class="pt-4">
                                        <button type="submit" class="btn btn-primary me-sm-3 me-1">Submit</button>
                                        <a href="{{ route('normal-login', ['id' => $role->id]) }}"
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
    <script>
        $(document).ready(function() {
            function addMemberRow() {
                const rowCount = $('#document-table tbody tr').length + 1;
                const newRow = `
            <tr>
                <td>${rowCount}</td>
                <td><input type="text" name="doc_name[]" class="form-control" required /></td>
                <td>
                    <select name="doc_type[]" required class="form-control">
                        <option value="standard">Standard</option>
                        <option value="blockchain">Blockchain</option>
                    </select>
                </td>
                <td>
                    <input type="file" name="documents[]" class="form-control"
                           accept=".pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png"
                           required />
                </td>
                <td><button type="button" class="btn btn-danger remove-row">Remove</button></td>
            </tr>`;
                $('#document-table tbody').append(newRow);
                updateSerialNumbers('#document-table');
                toggleRemoveButtons();
            }

            function updateSerialNumbers(tableId) {
                $(tableId + ' tbody tr').each(function(index) {
                    $(this).find('td:first').text(index + 1);
                });
            }

            function toggleRemoveButtons() {
                const rows = $('#document-table tbody tr');
                // prevent deleting the last remaining row
                rows.find('.remove-row').prop('disabled', rows.length === 1);
            }

            $('#doc-row').on('click', addMemberRow);

            $('body').on('click', '.remove-row', function() {
                $(this).closest('tr').remove();
                updateSerialNumbers('#document-table');
                toggleRemoveButtons();
                if ($('#document-table tbody tr').length === 0) addMemberRow();
            });

            // start with one row
            addMemberRow();
        });
    </script>
</body>

</html>
