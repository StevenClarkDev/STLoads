@extends('layout.app')
@section('content')
    <div>
        <div class="page-title">
            <div class="row">
                <div class="col-6">
                    <h4>Add Load</h4>
                </div>
                <div class="col-6">
                    <ol class="breadcrumb">
                        <li class="breadcrumb-item"><a href="index.html">
                                <svg class="stroke-icon">
                                    <use href="../assets/svg/icon-sprite.svg#stroke-home"></use>
                                </svg></a></li>
                        <li class="breadcrumb-item"> Add Load</li>
                        <li class="breadcrumb-item active"> Load Details</li>
                    </ol>
                </div>
            </div>
        </div>
    </div>

    <!-- Container-fluid starts-->
    <div>
        <div class="row">
            <div class="col-xl-12">
                <div class="card height-equal">
                    <div class="card-header">
                        <h4>Load Details</h4>
                    </div>
                    <div class="card-body">
                        <form class="row g-3 custom-input" action="{{ route('loads.store') }}" method="POST"
                            enctype="multipart/form-data">
                            @csrf
                            <div class="col-md-6">
                                <label class="form-label" for="title">Title</label>
                                <input class="form-control" id="title" name="title" type="text" required>
                                <input id="user_id" name="user_id" type="hidden" value="{{ $user_id }}">
                                <input id="role_id" name="role_id" type="hidden" value="{{ $roleId }}">
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="load_type">Load Type</label>
                                <select class="form-select" id="load_type" name="load_type_id" required>
                                    <option selected value="">Choose...</option>
                                    @foreach ($load_types as $load_type)
                                        <option value="{{ $load_type->id }}">{{ $load_type->name }}</option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="equipment_required">Equipment Required</label>
                                <select class="form-select" id="equipment_required" name="equipment_id" required>
                                    <option selected value="">Choose...</option>
                                    @foreach ($equipments as $equipment)
                                        <option value="{{ $equipment->id }}">{{ $equipment->name }}</option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="weight_unit">WT Unit</label>
                                <select class="form-select" id="weight_unit" name="weight_unit" required>
                                    <option selected value="">Choose...</option>
                                    <option value="LBS">LBS</option>
                                    <option value="KG">KG</option>
                                    <option value="MTON">MTON</option>
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="weight">Weight</label>
                                <input class="form-control" id="weight" type="number" name="weight" required>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="commodity_type">Commodity Type</label>
                                <select class="form-select" id="commodity_type" name="commodity_type_id" required>
                                    <option selected value="">Choose...</option>
                                    @foreach ($commodity_types as $commodity_type)
                                        <option value="{{ $commodity_type->id }}">{{ $commodity_type->name }}</option>
                                    @endforeach
                                </select>
                            </div>
                            {{-- <div class="col-md-6">
                                <label class="form-label" for="formFile1">Documents</label>
                                <input class="form-control" id="formFile1" type="file" name="documents"
                                    accept=".pdf,.jpg,.jpeg,.png,.webp,.doc,.docx" />
                                <div class="invalid-feedback">Invalid form file selected</div>
                            </div> --}}
                            <div class="col-md-6">
                                <div class="d-flex flex-row mt-4">
                                    <div class="form-check my-2 me-4">
                                        <input class="form-check-input" type="checkbox" id="is_hazardous"
                                            name="is_hazardous">
                                        <label for="is_hazardous">Is Hazardous Material?</label>
                                    </div>
                                    <div class="form-check my-2">
                                        <input class="form-check-input" type="checkbox" id="is_temperature_controlled"
                                            name="is_temperature_controlled">
                                        <label for="is_temperature_controlled">Is Temperature Controlled?</label>
                                    </div>
                                </div>
                            </div>
                            <div class="col-mb-6 mb-3">
                                <label class="form-label" for="validationTextarea">Special Instructions</label>
                                <textarea class="form-control" id="validationTextarea" name="special_instructions"
                                    placeholder="Enter your Special Instructions"></textarea>
                                <div class="invalid-feedback">Please enter a message in the textarea.</div>
                            </div>

                            <div class="col-xl-12">
                                <div class="card">
                                    <div class="card-header d-flex">
                                        <h4 class="mb-0">Documents</h4>
                                        <button type="button" class="btn btn-primary btn-sm" id="doc-row">
                                            <i class="bi bi-plus-lg"></i> Add
                                        </button>
                                    </div>
                                    <div class="card-body">
                                        <div class="table-responsive">
                                            <table class="table table-bordered" id="document-table">
                                                <thead>
                                                    <tr>
                                                        <th>#</th>
                                                        <th>Document Name</th>
                                                        <th>Document Type</th>
                                                        <th>Document</th>
                                                        <th>Action</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <!-- Load Legs Table -->
                            <div class="col-xl-12">
                                <div class="card">
                                    <div class="card-header d-flex">
                                        <h4 class="mb-0">Load Legs</h4>
                                        @if ($roleId == 5)
                                            <button type="button" class="btn btn-primary btn-sm" id="add-load_legs-row">
                                                <i class="bi bi-plus-lg"></i> Add
                                            </button>
                                        @endif
                                    </div>
                                    <div class="card-body">
                                        <div class="table-responsive">
                                            <table class="table table-bordered" id="load_legs-table">
                                                <thead class="table-light">
                                                    <tr>
                                                        <th style="width: 40px;">S.No</th>
                                                        <th style="width: 300px;">Pickup Location</th>
                                                        <th style="width: 300px;">Delivery Location</th>
                                                        <th style="width: 150px;">Pickup Date</th>
                                                        <th style="width: 150px;">Delivery Date</th>
                                                        <th style="width: 120px;">Bid Status</th>
                                                        <th style="width: 100px;">Price</th>
                                                        @if ($roleId == 5)
                                                            <th style="width: 80px;">Action</th>
                                                        @endif
                                                    </tr>
                                                </thead>
                                                <tbody></tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="col-12">
                                <button class="btn btn-primary" type="submit">Save Details</button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <!-- Container-fluid Ends-->
@endsection



<script src="{{ url('assets/js/jquery.min.js') }}"></script>

<script>
    $(function() {
        const canEditLegs = {{ $roleId == 5 ? 'true' : 'false' }};

        function renumberRows() {
            $('#load_legs-table tbody tr').each(function(i) {
                $(this).find('input.leg-id').val(i + 1);
            });
        }

        // ----- Row template (pure JS; inject prebuilt options) -----
        function rowTemplate() {
            return `
        <tr>
          <td style="width: 40px;"><input type="text" name="leg_id[]" class="form-control leg-id" value="" readonly /></td>

          <td style="width: 300px;">
            <input type="text" name="pickup_location_address[]" class="form-control location-autocomplete pickup-location" 
                   placeholder="Search pickup address..." required>
            <input type="hidden" name="pickup_city[]" class="pickup-city-hidden">
            <input type="hidden" name="pickup_country[]" class="pickup-country-hidden">
          </td>

          <td style="width: 300px;">
            <input type="text" name="delivery_location_address[]" class="form-control location-autocomplete delivery-location" 
                   placeholder="Search delivery address..." required>
            <input type="hidden" name="delivery_city[]" class="delivery-city-hidden">
            <input type="hidden" name="delivery_country[]" class="delivery-country-hidden">
          </td>

          <td>
            <div class="input-group flatpicker-calender">
              <input class="form-control datepicker" name="pickup_date[]" type="text" required>
            </div>
          </td>

          <td>
            <div class="input-group flatpicker-calender">
              <input class="form-control datepicker" name="delivery_date[]" type="date" required>
            </div>
          </td>

          <td>
            <select name="bid_status[]" class="form-select" required>
              <option value="Fixed">Fixed</option>
              <option value="Open">Open</option>
            </select>
          </td>

          <td><input type="number" min="0" name="price[]" class="form-control" required /></td>

          ${canEditLegs ? `
            <td>
              <button type="button" class="btn-remove-icon remove-row-load_legs" title="Remove">
                <i class="bi bi-trash"></i>
              </button>
            </td>
          ` : ``}
        </tr>
      `;
        }

        function addRow() {
            const $row = $(rowTemplate());
            $('#load_legs-table tbody').append($row);
            renumberRows();
            // Initialize Google Maps Autocomplete for newly added row
            initializeAutocomplete($row.find('.location-autocomplete'));
        }

        // ----- Events -----
        $('#add-load_legs-row').on('click', addRow);

        $('#load_legs-table').on('click', '.remove-row-load_legs', function() {
            $(this).closest('tr').remove();
            renumberRows();
        });

        // ----- Init -----
        addRow();
    });
</script>
<script>
    $(document).ready(function() {
        function addMemberRow() {
            const rowCount = $('#document-table tbody tr').length + 1;
            const newRow = `
            <tr>
                <td>${rowCount}</td>
                <td><input type="text" name="doc_name[]" class="form-control" /></td>
                <td>
                    <select name="doc_type[]" class="form-control">
                        <option value="standard">Standard</option>
                        <option value="blockchain">Blockchain</option>
                    </select>
                </td>
                <td>
                    <input type="file" name="documents[]" class="form-control"
                           accept=".pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png" />
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

<!-- Google Maps Custom Styles -->
<style>
    /* Enhanced spacing for location inputs */
    .location-autocomplete {
        padding: 12px 16px !important;
        font-size: 15px !important;
        line-height: 1.5 !important;
        border-radius: 6px !important;
        border: 1px solid #d1d5db !important;
        transition: all 0.2s ease !important;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05) !important;
    }
    
    .location-autocomplete:focus {
        border-color: #3b82f6 !important;
        box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1) !important;
        outline: none !important;
    }
    
    .location-autocomplete::placeholder {
        color: #9ca3af !important;
        font-weight: 400 !important;
    }
    
    /* Hide Google branding */
    .pac-container:after {
        display: none !important;
    }
    
    .pac-logo:after {
        display: none !important;
    }
    
    /* Enhanced autocomplete dropdown styling */
    .pac-container {
        background-color: #fff;
        border: 1px solid #e5e7eb;
        border-radius: 8px;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
        margin-top: 4px;
        padding: 4px 0;
        font-family: inherit;
        z-index: 9999 !important;
    }
    
    .pac-item {
        padding: 10px 16px;
        font-size: 14px;
        line-height: 1.5;
        cursor: pointer;
        border-top: none;
        transition: background-color 0.15s ease;
    }
    
    .pac-item:hover {
        background-color: #f3f4f6;
    }
    
    .pac-item-selected,
    .pac-item-selected:hover {
        background-color: #eff6ff;
    }
    
    .pac-item-query {
        font-size: 14px;
        font-weight: 600;
        color: #1f2937;
        padding-right: 4px;
    }
    
    .pac-matched {
        font-weight: 700;
        color: #2563eb;
    }
    
    .pac-icon {
        margin-top: 2px;
        margin-right: 12px;
        width: 18px;
        height: 18px;
        background-size: 18px;
    }
    
    /* Better spacing for table cells */
    #load_legs-table td {
        padding: 12px 8px !important;
        vertical-align: middle;
    }
    
    #load_legs-table .form-control,
    #load_legs-table .form-select {
        margin: 0;
    }
    
    /* Loading indicator for autocomplete */
    .location-autocomplete.loading {
        background-image: url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAiIGhlaWdodD0iMjAiIHZpZXdCb3g9IjAgMCAyMCAyMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48Y2lyY2xlIGN4PSIxMCIgY3k9IjEwIiByPSI4IiBzdHJva2U9IiMzYjgyZjYiIHN0cm9rZS13aWR0aD0iMiIgZmlsbD0ibm9uZSIgc3Ryb2tlLWRhc2hhcnJheT0iMTAgNTAiPjxhbmltYXRlVHJhbnNmb3JtIGF0dHJpYnV0ZU5hbWU9InRyYW5zZm9ybSIgdHlwZT0icm90YXRlIiBmcm9tPSIwIDEwIDEwIiB0bz0iMzYwIDEwIDEwIiBkdXI9IjFzIiByZXBlYXRDb3VudD0iaW5kZWZpbml0ZSIvPjwvY2lyY2xlPjwvc3ZnPg==');
        background-repeat: no-repeat;
        background-position: right 12px center;
        background-size: 20px 20px;
    }
</style>

<!-- Google Maps Places API -->
<script src="https://maps.googleapis.com/maps/api/js?key={{ env('GOOGLE_MAPS_API_KEY') }}&libraries=places&callback=initMap" async defer></script>

<script>
    // Global function for Google Maps callback
    function initMap() {
        // Initialize autocomplete for all existing location fields
        initializeAutocomplete($('.location-autocomplete'));
    }

    // Function to initialize Google Places Autocomplete
    function initializeAutocomplete(elements) {
        if (typeof google === 'undefined' || !google.maps || !google.maps.places) {
            console.warn('Google Maps API not loaded yet. Retrying...');
            setTimeout(function() {
                initializeAutocomplete(elements);
            }, 500);
            return;
        }

        elements.each(function() {
            const input = this;
            
            // Skip if already initialized
            if ($(input).data('autocomplete-initialized')) {
                return;
            }

            // Enhanced autocomplete configuration with US/Canada restriction
            const autocompleteOptions = {
                types: ['geocode', 'establishment'],  // More flexible search (addresses, cities, landmarks)
                fields: ['formatted_address', 'geometry', 'name', 'address_components', 'place_id'],
                componentRestrictions: { country: ['us', 'ca'] }  // Restrict to US and Canada only
            };
            
            // Store address components in hidden inputs
            $(input).data('address-components', null);

            // Try to get user's location for biasing results
            if (navigator.geolocation) {
                navigator.geolocation.getCurrentPosition(
                    function(position) {
                        const userLocation = new google.maps.LatLng(
                            position.coords.latitude,
                            position.coords.longitude
                        );
                        
                        // Create circle for biasing (500km radius)
                        autocompleteOptions.bounds = {
                            north: position.coords.latitude + 4.5,
                            south: position.coords.latitude - 4.5,
                            east: position.coords.longitude + 4.5,
                            west: position.coords.longitude - 4.5
                        };
                        autocompleteOptions.strictBounds = false;  // Don't strictly enforce bounds, just bias
                    },
                    function(error) {
                        console.log('Geolocation not available, using default biasing');
                    }
                );
            }

            // Create the autocomplete object
            const autocomplete = new google.maps.places.Autocomplete(input, autocompleteOptions);
            
            // Add loading indicator when typing
            let typingTimer;
            $(input).on('input', function() {
                clearTimeout(typingTimer);
                $(this).addClass('loading');
                typingTimer = setTimeout(function() {
                    $(input).removeClass('loading');
                }, 300);
            });

            // When user selects a place from dropdown
            autocomplete.addListener('place_changed', function() {
                $(input).removeClass('loading');
                const place = autocomplete.getPlace();
                
                if (!place.geometry) {
                    console.log("No details available for: '" + place.name + "'");
                    return;
                }

                // Update the input value with formatted address
                $(input).val(place.formatted_address);
                
                // Extract city and country from address_components
                let city = '';
                let country = '';
                let countryShort = '';
                
                if (place.address_components) {
                    place.address_components.forEach(component => {
                        if (component.types.includes('locality')) {
                            city = component.long_name;
                        } else if (component.types.includes('administrative_area_level_1') && !city) {
                            city = component.long_name; // Fallback to state/province
                        }
                        if (component.types.includes('country')) {
                            country = component.long_name;
                            countryShort = component.short_name;
                        }
                    });
                }
                
                // Store address components in data attribute
                $(input).data('address-components', {
                    city: city,
                    country: country,
                    countryShort: countryShort,
                    lat: place.geometry.location.lat(),
                    lng: place.geometry.location.lng()
                });
                
                // Update hidden inputs for city and country
                const $row = $(input).closest('tr');
                if ($(input).hasClass('pickup-location')) {
                    $row.find('.pickup-city-hidden').val(city);
                    $row.find('.pickup-country-hidden').val(country);
                } else if ($(input).hasClass('delivery-location')) {
                    $row.find('.delivery-city-hidden').val(city);
                    $row.find('.delivery-country-hidden').val(country);
                }
                
                console.log('Selected location:', {
                    address: place.formatted_address,
                    city: city,
                    country: country,
                    lat: place.geometry.location.lat(),
                    lng: place.geometry.location.lng()
                });
            });

            // Mark as initialized
            $(input).data('autocomplete-initialized', true);
        });
    }
</script>
