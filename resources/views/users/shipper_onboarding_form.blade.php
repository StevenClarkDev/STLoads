@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 1100px; width: 100%;">
        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 30%;">
        </div>
        <div class="mb-5">
            <h4 class="text-center col-blue mb-2">Professional Shippers</h4>
            <h2 class="text-center mb-2">Sign <span class="col-blue"> Up</span></h2>
            <p class="text-muted text-center mb-2">Please complete the form below to get started. Each section will guide
                you
                through
                essential details about your shipping needs.</p>
        </div>
        <div class="numbering-wizard mb-4">
            <div class="d-flex justify-content-between position-relative">
                <div class="step active" data-step="1">1</div>
                <div class="step" data-step="2">2</div>
                <div class="step" data-step="3">3</div>
                <div class="step" data-step="4">4</div>
                <div class="step" data-step="5">5</div>
                <div class="step" data-step="6">6</div>
                <div class="progress-bar position-absolute"></div>
            </div>
            <div class="d-flex justify-content-between mt-2">
                <div class="step-label active" data-step="1">Step 1</div>
                <div class="step-label" data-step="2">Step 2</div>
                <div class="step-label" data-step="3">Step 3</div>
                <div class="step-label" data-step="4">Step 4</div>
                <div class="step-label" data-step="5">Step 5</div>
                <div class="step-label" data-step="6">Step 6</div>
            </div>
        </div>

        <form class="row g-3" action="{{ route('onboarding-form-save-shipper', $user->id) }}" method="POST"
            enctype="multipart/form-data" id="registrationForm">
            @csrf

            <!-- All step contents wrapped in a scrollable container -->
            <div class="step-container" style="height: 300px; overflow-y: auto;">
                <!-- Step 1: Basic Info -->
                <div class="step-content active" data-step="1">
                    <div class="my-3">
                        <h4 class="text-center">Company & Contact Information</h4>
                    </div>
                    <hr>
                    <div class="col-md-12 d-flex gap-2 mb-2">
                        <div class="col-md-6 position-relative">
                            <label>Business or Personal Name</label>
                            <div class="input-group">
                                <input id="company_name" class="form-control pe-5 rounded-2" type="text"
                                    name="company_name" placeholder="Enter your Business or Personal Name" value="{{ old('company_name') }}" required>
                                <i id="name-icon"
                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                            </div>
                        </div>
                        <div class="col-md-6 position-relative">
                            <label>Business Address (Pickup Location)</label>
                            <div class="input-group">
                                <input id="company_address" class="form-control pe-5 rounded-2" type="text"
                                    name="company_address" placeholder="Enter your Business Address (Pickup Location)" value="{{ old('company_address') }}"
                                    required>
                                <i id="name-icon"
                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-12 d-flex gap-2 mb-2">
                        <div class="col-md-6 position-relative">
                            <label>Business Type</label>
                            <div class="input-group">
                                <input id="business_type" class="form-control pe-5 rounded-2" type="text"
                                    name="business_type" placeholder="Business Type (e.g., E-commerce, Manufacturer)" value="{{ old('business_type') }}"
                                    required>
                                <i id="name-icon"
                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                            </div>
                        </div>
                        <div class="col-md-6 position-relative">
                            <label>Website</label>
                            <div class="input-group">
                                <input id="website" class="form-control pe-5 rounded-2" type="text" name="website" value="{{ old('website') }}"
                                    placeholder="Website (if applicable)">
                                <i id="name-icon"
                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-12 d-flex gap-2 mb-2">
                        <div class="col-md-6">
                            <div class="form-floating form-floating-outline">
                                <input type="file" accept=".jpeg, .jpg, .png, .pdf" name="cnic_front" id="cnic_front"
                                    class="form-control" />
                                <label>CNIC Front</label>
                            </div>
                        </div>
                        <div class="col-md-6">
                            <div class="form-floating form-floating-outline">
                                <input type="file" accept=".jpeg, .jpg, .png, .pdf" name="cnic_back" id="cnic_back"
                                    class="form-control" />
                                <label>CNIC Back</label>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Step 2: Additional Info -->
                <div class="step-content" data-step="2">
                    <div class="my-3">
                        <h4 class="text-center">Shipment Details</h4>
                    </div>
                    <hr>
                    <div class="mb-2 row">
                        <div class="col-md-12">
                            <label>Types of Goods Being Shipped:</label>
                            <div class="form-check-size mb-5">
                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="types_of_goods_being_shipped-1" type="checkbox"
                                        name="types_of_goods_being_shipped[]" value="retail_merchandise">
                                    <label class="form-check-label" for="types_of_goods_being_shipped-1">Retail
                                        Merchandise</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="types_of_goods_being_shipped-2" type="checkbox"
                                        name="types_of_goods_being_shipped[]" value="palletized_freight">
                                    <label class="form-check-label" for="types_of_goods_being_shipped-2">Palletized
                                        Freight</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="types_of_goods_being_shipped-3" type="checkbox"
                                        name="types_of_goods_being_shipped[]" value="documents_parcels">
                                    <label class="form-check-label"
                                        for="types_of_goods_being_shipped-3">Documents/Parcels</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="types_of_goods_being_shipped-4" type="checkbox"
                                        name="types_of_goods_being_shipped[]" value="food_and_beverages">
                                    <label class="form-check-label" for="types_of_goods_being_shipped-4">Food &
                                        Beverages</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="types_of_goods_being_shipped-5" type="checkbox"
                                        name="types_of_goods_being_shipped[]" value="hazardous_materials">
                                    <label class="form-check-label" for="types_of_goods_being_shipped-5">Hazardous
                                        Materials</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="types_of_goods_being_shipped-6" type="checkbox"
                                        name="types_of_goods_being_shipped[]" value="furniture">
                                    <label class="form-check-label" for="types_of_goods_being_shipped-6">Furniture</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="types_of_goods_being_shipped-7" type="checkbox"
                                        name="types_of_goods_being_shipped[]" value="medical_supplies">
                                    <label class="form-check-label" for="types_of_goods_being_shipped-7">Medical
                                        Supplies</label>
                                </div>

                                <!-- Other with input -->
                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="types_of_goods_being_shipped-8" type="checkbox"
                                        name="types_of_goods_being_shipped[]" value="other">
                                    <label class="form-check-label" for="types_of_goods_being_shipped-8">Other</label>
                                </div>

                                <!-- Hidden input initially -->
                                <div id="otherGoodsInput" style="display:none; margin-top:10px;">
                                    <input type="text" class="form-control" id="other_goods" name="other_goods"
                                        placeholder="Please specify other goods">
                                </div>
                            </div>
                        </div>

                        <div class="col-md-6">
                            <label>Shipments per Week</label>
                            <div class="input-group">
                                <input id="shipments_per_week" class="form-control pe-5 rounded-2" type="number"
                                    name="shipments_per_week" placeholder="Average Number of Shipments per Week" required>
                            </div>
                        </div>
                        <div class="col-md-6">
                            <label>Volume/Weight per Shipment</label>
                            <div class="input-group">
                                <input id="volume_or_weight_per_shipment" class="form-control pe-5 rounded-2"
                                    type="text" name="volume_or_weight_per_shipment"
                                    placeholder="Estimated Volume/Weight per Shipment" required>
                            </div>
                        </div>

                        <div class="col-md-12 mt-3">
                            <label>Packaging Type:</label>
                            <div class="form-check-size mt-3">
                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="packaging_type-1" type="checkbox"
                                        name="packaging_type[]" value="boxes">
                                    <label class="form-check-label" for="packaging_type-1">Boxes</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="packaging_type-2" type="checkbox"
                                        name="packaging_type[]" value="pallets">
                                    <label class="form-check-label" for="packaging_type-2">Pallets</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="packaging_type-3" type="checkbox"
                                        name="packaging_type[]" value="crates">
                                    <label class="form-check-label" for="packaging_type-3">Crates</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="packaging_type-4" type="checkbox"
                                        name="packaging_type[]" value="envelopes">
                                    <label class="form-check-label" for="packaging_type-4">Envelopes</label>
                                </div>

                                <div class="form-check form-check-inline mb-0">
                                    <input class="checkbox_animated" id="packaging_type-5" type="checkbox"
                                        name="packaging_type[]" value="mixed">
                                    <label class="form-check-label" for="packaging_type-5">Mixed</label>
                                </div>
                            </div>
                        </div>

                    </div>
                </div>

                <!-- Step 3: Profile Upload -->
                <div class="step-content" data-step="3">
                    <div class="my-3">
                        <h4 class="text-center">Delivery Requirements</h4>
                    </div>
                    <hr>
                    <div class="col-md-12">
                        <label class="mb-2">Types of Delivery Services Needed:</label>
                        <div class="form-check-size mb-5">
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-1" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="local_sameDay_delivery">
                                <label class="form-check-label" for="types_of_delivery_services_needed-1">Local Same-Day
                                    Delivery</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-2" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="nextDay_delivery">
                                <label class="form-check-label" for="types_of_delivery_services_needed-2">Next-Day
                                    Delivery</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-3" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="scheduled_pickups">
                                <label class="form-check-label" for="types_of_delivery_services_needed-3">Scheduled
                                    Pickups</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-4" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="final_mile_delivery">
                                <label class="form-check-label" for="types_of_delivery_services_needed-4">Final Mile
                                    Delivery</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-5" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="white_glove_delivery">
                                <label class="form-check-label" for="types_of_delivery_services_needed-5">White Glove
                                    Delivery</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-6" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="liftgate_required">
                                <label class="form-check-label" for="types_of_delivery_services_needed-6">Liftgate
                                    Required</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-7" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="weekend_or_holiday_deliveries">
                                <label class="form-check-label" for="types_of_delivery_services_needed-7">Weekend/Holiday
                                    Deliveries</label>
                            </div>

                            <!-- Other with input -->
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-8" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="temperature_controlled">
                                <label class="form-check-label"
                                    for="types_of_delivery_services_needed-8">Temperature-Controlled</label>
                            </div>
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="types_of_delivery_services_needed-9" type="checkbox"
                                    name="types_of_delivery_services_needed[]" value="special_handling">
                                <label class="form-check-label" for="types_of_delivery_services_needed-9">Special
                                    Handling</label>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6 select2-primary">
                        <label for="preferred_pickup_days">Preferred Pickup Days:</label>
                        <div class="form-floating form-floating-outline">
                            <select id="preferred_pickup_days" class="select2 form-select" name="preferred_pickup_days[]"
                                multiple>
                                <option value="monday">Monday</option>
                                <option value="tuesday">Tuesday</option>
                                <option value="wednesday">Wednesday</option>
                                <option value="thursday">Thursday</option>
                                <option value="friday">Friday</option>
                                <option value="saturday">Saturday</option>
                                <option value="sunday">Sunday</option>
                            </select>
                        </div>
                    </div>
                    <div class="col-md-12">
                        <label class="col-xxl-3 box-col-12 text-start">Preferred Pickup Time Range</label>
                        <div class="row">
                            <div class="col-xxl-6 box-col-6">
                                <div class="input-group">
                                    <input class="form-control" id="time-picker" type="time" value="12:00"
                                        name="preferred_pickup_from_time">
                                </div>
                            </div>
                            <div class="col-xxl-6 box-col-6">
                                <div class="input-group">
                                    <input class="form-control" id="time-picker" type="time" value="12:00"
                                        name="preferred_pickup_to_time">
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-12">
                        <label class="mb-2 mt-2">Typical Delivery Destinations:</label>
                        <div class="form-check-size mb-5">
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="typical_delivery_destinations-1" type="checkbox"
                                    name="typical_delivery_destinations[]" value="local">
                                <label class="form-check-label" for="typical_delivery_destinations-1">Local</label>
                            </div>
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="typical_delivery_destinations-2" type="checkbox"
                                    name="typical_delivery_destinations[]" value="regional">
                                <label class="form-check-label" for="typical_delivery_destinations-2">Regional</label>
                            </div>
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="typical_delivery_destinations-3" type="checkbox"
                                    name="typical_delivery_destinations[]" value="national">
                                <label class="form-check-label" for="typical_delivery_destinations-3">National</label>
                            </div>
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="typical_delivery_destinations-4" type="checkbox"
                                    name="typical_delivery_destinations[]" value="international">
                                <label class="form-check-label"
                                    for="typical_delivery_destinations-4">International</label>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Step 4: Review and Submit -->
                <div class="step-content" data-step="4">
                    <div class="text-center mb-4">
                        <h4 class="mb-2">Account & Billing Info</h4>
                    </div>

                    <hr>
                    <div class="col-md-12">
                        <label class="mb-2">Do You Have an Existing Logistics Provider?</label>
                        <div class="form-check-size">
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="radioinline1" type="radio"
                                    name="logistics_provider" value="yes">
                                <label class="form-check-label mb-0" for="radioinline1">Yes</label>
                            </div>
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="radioinline2" type="radio"
                                    name="logistics_provider" value="no">
                                <label class="form-check-label mb-0" for="radioinline2">No</label>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-12">
                        <label>Preferred Payment Method:</label>
                        <div class="form-check-size mb-5">
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="preferred_payment_method-1" type="checkbox"
                                    name="preferred_payment_method[]" value="credit_card">
                                <label class="form-check-label" for="preferred_payment_method-1">Credit Card</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="preferred_payment_method-2" type="checkbox"
                                    name="preferred_payment_method[]" value="ach_or_bank_transfer">
                                <label class="form-check-label" for="preferred_payment_method-2">ACH/Bank Transfer</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="preferred_payment_method-3" type="checkbox"
                                    name="preferred_payment_method[]" value="paypal">
                                <label class="form-check-label" for="preferred_payment_method-3">PayPal</label>
                            </div>

                            <!-- Other with input -->
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="preferred_payment_method-4" type="checkbox"
                                    name="preferred_payment_method[]" value="other">
                                <label class="form-check-label" for="preferred_payment_method-4">Other</label>
                            </div>

                            <!-- Hidden input initially -->
                            <div id="otherPaymentInput" style="display:none; margin-top:10px;">
                                <input type="text" class="form-control" id="other_payment" name="other_payment"
                                    placeholder="Please specify other payment method">
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6">
                        <label>Billing Contact Name</label>
                        <div class="input-group">
                            <input id="billing_contact_name" class="form-control pe-5 rounded-2" type="text"
                                name="billing_contact_name" placeholder="Enter your Billing Contact Name">
                        </div>
                    </div>
                    <div class="col-md-6">
                        <label>Billing Email Address</label>
                        <div class="input-group">
                            <input id="billing_email_address" class="form-control pe-5 rounded-2" type="text"
                                name="billing_email_address" placeholder="Enter your Billing Email Address">
                        </div>
                    </div>
                    <div class="col-md-6">
                        <label>Tax ID</label>
                        <div class="input-group">
                            <input id="tax_id" class="form-control pe-5 rounded-2" type="text" name="tax_id"
                                placeholder="Enter Tax ID">
                        </div>
                    </div>
                    <div class="col-md-12">
                        <label class="mb-2">Invoice Frequency:</label>
                        <div class="form-check-size">
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="invoice_frequency1" type="radio"
                                    name="invoice_frequency" value="Per Shipment">
                                <label class="form-check-label mb-0" for="invoice_frequency1">Per Shipment</label>
                            </div>
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="invoice_frequency2" type="radio"
                                    name="invoice_frequency" value="Weekly">
                                <label class="form-check-label mb-0" for="invoice_frequency2">Weekly</label>
                            </div>
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="invoice_frequency3" type="radio"
                                    name="invoice_frequency" value="Monthly">
                                <label class="form-check-label mb-0" for="invoice_frequency3">Monthly</label>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="step-content" data-step="5">
                    <div class="text-center mb-4">
                        <h4 class="mb-2">Additional Preferences</h4>
                    </div>

                    <hr>
                    <div class="col-md-6">
                        <label class="mb-2">Require Shipment Tracking?</label>
                        <div class="form-check-size">
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="shipment_tracking1" type="radio"
                                    name="shipment_tracking" value="Yes">
                                <label class="form-check-label mb-0" for="shipment_tracking1">Yes</label>
                            </div>
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="shipment_tracking2" type="radio"
                                    name="shipment_tracking" value="No">
                                <label class="form-check-label mb-0" for="shipment_tracking2">No</label>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6">
                        <label class="mb-2">Pickup Materials Supplied?</label>
                        <div class="form-check-size">
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="pickup_materials_supplied1" type="radio"
                                    name="pickup_materials_supplied" value="Yes">
                                <label class="form-check-label mb-0" for="pickup_materials_supplied1">Yes</label>
                            </div>
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="pickup_materials_supplied2" type="radio"
                                    name="pickup_materials_supplied" value="No">
                                <label class="form-check-label mb-0" for="pickup_materials_supplied2">No</label>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6">
                        <label class="mb-2">Schedule a Demo or Onboarding Call?</label>
                        <div class="form-check-size">
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="demo_or_onboarding_call1" type="radio"
                                    name="demo_or_onboarding_call" value="Yes">
                                <label class="form-check-label mb-0" for="demo_or_onboarding_call1">Yes</label>
                            </div>
                            <div class="form-check form-check-inline radio radio-primary">
                                <input class="form-check-input" id="demo_or_onboarding_call2" type="radio"
                                    name="demo_or_onboarding_call" value="No">
                                <label class="form-check-label mb-0" for="demo_or_onboarding_call2">No</label>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-12">
                        <label>Preferred Communication Method:</label>
                        <div class="form-check-size mb-5">
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="preferred_communication_method-1" type="checkbox"
                                    name="preferred_communication_method[]" value="email">
                                <label class="form-check-label" for="preferred_communication_method-1">Email</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="preferred_communication_method-2" type="checkbox"
                                    name="preferred_communication_method[]" value="phone">
                                <label class="form-check-label" for="preferred_communication_method-2">Phone</label>
                            </div>

                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="preferred_communication_method-3" type="checkbox"
                                    name="preferred_communication_method[]" value="sms">
                                <label class="form-check-label" for="preferred_communication_method-3">SMS</label>
                            </div>
                            <div class="form-check form-check-inline mb-0">
                                <input class="checkbox_animated" id="preferred_communication_method-4" type="checkbox"
                                    name="preferred_communication_method[]" value="in_app_messaging">
                                <label class="form-check-label" for="preferred_communication_method-4">In-App
                                    Messaging</label>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="step-content" data-step="6">
                    <div class="text-center mb-4">
                        <h4 class="mb-2">Notes or Special Instructions</h4>
                    </div>

                    <hr>
                    <div class="col-md-12">
                        <textarea class="form-control" name="special_notes"
                            placeholder="Special logistics needs, hours of operation, dock access info, etc." style="height: 100px"></textarea>
                    </div>
                </div>
            </div>

            <!-- Navigation Buttons -->
            <div class="col-12 d-flex justify-content-between mt-4">
                <button type="button" class="btn btn-outline-secondary" id="prevBtn" disabled>Back</button>
                <button type="button" class="btn btn-primary" id="nextBtn">Next</button>
                <button type="submit" class="btn btn-success" id="submitBtn" style="display: none;">Submit</button>
            </div>

            <!-- Laravel error/success messages -->
            @if ($errors->any())
                <div class="col-12 text-danger text-center mt-2">
                    {{ $errors->first() }}
                </div>
            @endif

            @if (session('success'))
                <div class="col-12 text-success text-center mt-2">
                    {{ session('success') }}
                </div>
            @endif
        </form>
    </div>

    <style>
        .numbering-wizard {
            margin-bottom: 2rem;
        }

        .col-blue {
            color: #00ADF1;
        }

        .step {
            width: 40px;
            height: 40px;
            border-radius: 50%;
            background-color: #e9ecef;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: bold;
            z-index: 2;
        }

        .step.active {
            background-color: #1F537B;
            color: white;
        }

        .step-label {
            font-size: 0.9rem;
            color: #6c757d;
        }

        .step-label.active {
            font-weight: bold;
            color: #1F537B;
        }

        .progress-bar {
            height: 2px;
            background-color: #e9ecef;
            top: 20px;
            left: 20px;
            right: 20px;
            z-index: 1;
        }

        .step-content {
            display: none;
        }

        .step-content.active {
            display: block;
        }

        .review-card {
            transition: all 0.3s ease;
            border-width: 1px;
            border-color: #dee2e6 !important;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
        }

        .review-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
        }

        .card-header {
            border-bottom: 1px solid #dee2e6;
            text-align: center;
            /* background-color: #f8f9fa !important; */
        }

        .step-container {
            scrollbar-width: thin;
            scrollbar-color: #1F537B #f1f1f1;
        }

        .step-container::-webkit-scrollbar {
            width: 6px;
        }

        .step-container::-webkit-scrollbar-track {
            background: #f1f1f1;
        }

        .step-container::-webkit-scrollbar-thumb {
            background-color: #1F537B;
            border-radius: 6px;
        }

        .fa-check-circle.text-success {
            color: #28a745 !important;
        }

        .border-light {
            border-color: #ecf3fa !important;
        }
    </style>

    <script src="{{ url('assets/js/jquery.min.js') }}"></script>
    <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11.22.4/dist/sweetalert2.all.min.js"></script>

    <script>
        document.addEventListener('DOMContentLoaded', function() {
            const otherCheckbox = document.getElementById("types_of_goods_being_shipped-8");
            const otherPaymentCheckbox = document.getElementById("preferred_payment_method-4");
            const otherInput = document.getElementById("otherGoodsInput");
            const otherInputPayment = document.getElementById("otherPaymentInput");

            otherCheckbox.addEventListener("change", function() {
                if (this.checked) {
                    otherInput.style.display = "block";
                    document.getElementById("other_goods").focus();
                } else {
                    otherInput.style.display = "none";
                    document.getElementById("other_goods").value = "";
                }
            });
            otherPaymentCheckbox.addEventListener("change", function() {
                if (this.checked) {
                    otherInputPayment.style.display = "block";
                    document.getElementById("other_payment").focus();
                } else {
                    otherInputPayment.style.display = "none";
                    document.getElementById("other_payment").value = "";
                }
            });
            const steps = document.querySelectorAll('.step-content');
            const stepIndicators = document.querySelectorAll('.step');
            const stepLabels = document.querySelectorAll('.step-label');
            const prevBtn = document.getElementById('prevBtn');
            const nextBtn = document.getElementById('nextBtn');
            const submitBtn = document.getElementById('submitBtn');
            let currentStep = 1;

            // Initialize progress bar
            updateProgressBar();

            // Next button click handler
            nextBtn.addEventListener('click', function() {
                if (validateStep(currentStep)) {
                    if (currentStep < steps.length) {
                        // Move to next step
                        document.querySelector(`.step-content[data-step="${currentStep}"]`).classList
                            .remove('active');
                        document.querySelector(`.step[data-step="${currentStep}"]`).classList.remove(
                            'active');
                        document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.remove(
                            'active');

                        currentStep++;

                        document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.add(
                            'active');
                        document.querySelector(`.step[data-step="${currentStep}"]`).classList.add('active');
                        document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.add(
                            'active');

                        updateButtons();
                        updateProgressBar();

                    }
                }
            });

            // Previous button click handler
            prevBtn.addEventListener('click', function() {
                if (currentStep > 1) {
                    document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.remove(
                        'active');
                    document.querySelector(`.step[data-step="${currentStep}"]`).classList.remove('active');
                    document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.remove(
                        'active');

                    currentStep--;

                    document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.add(
                        'active');
                    document.querySelector(`.step[data-step="${currentStep}"]`).classList.add('active');
                    document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.add(
                        'active');

                    updateButtons();
                    updateProgressBar();
                }
            });

            // Update button states based on current step
            function updateButtons() {
                prevBtn.disabled = currentStep === 1;
                nextBtn.style.display = currentStep === steps.length ? 'none' : 'block';
                submitBtn.style.display = currentStep === steps.length ? 'block' : 'none';
            }

            // Update progress bar
            function updateProgressBar() {
                const progressBar = document.querySelector('.progress-bar');
                const progressPercentage = ((currentStep - 1) / (steps.length - 1)) * 100;
                progressBar.style.background =
                    `linear-gradient(to right, #1F537B ${progressPercentage}%, #e9ecef ${progressPercentage}%)`;
            }

            // Validate current step before proceeding
            // Validate current step before proceeding
            function validateStep(step) {
                let isValid = true;

                if (step === 1) {
                    const company_name = document.getElementById('company_name').value;
                    const company_address = document.getElementById('company_address').value;
                    const business_type = document.getElementById('business_type').value;
                    const cnic_front = document.getElementById('cnic_front');
                    const cnic_back = document.getElementById('cnic_back');

                    if (!company_name || !company_address || !business_type || !cnic_front.files.length || !
                        cnic_back.files.length) {
                        Swal.fire({
                            position: 'center',
                            icon: 'error',
                            title: 'Error',
                            text: 'Please fill all required fields',
                            showConfirmButton: false,
                            showCloseButton: true,
                            allowOutsideClick: false,
                            allowEscapeKey: false,
                            backdrop: true,
                        });
                        isValid = false;
                    }
                } else if (step === 2) {
                    const typesOfGoods = document.querySelectorAll(
                        'input[name="types_of_goods_being_shipped[]"]:checked').length;
                    const shipmentsPerWeek = document.getElementById('shipments_per_week').value;
                    const volumeOrWeight = document.getElementById('volume_or_weight_per_shipment').value;

                    if (typesOfGoods === 0 || !shipmentsPerWeek || !volumeOrWeight) {
                        Swal.fire({
                            position: 'center',
                            icon: 'error',
                            title: 'Error',
                            text: 'Please fill all required fields',
                            showConfirmButton: false,
                            showCloseButton: true,
                            allowOutsideClick: false,
                            allowEscapeKey: false,
                            backdrop: true,
                        });
                        isValid = false;
                    }
                } else if (step === 3) {
                    const deliveryServices = document.querySelectorAll(
                        'input[name="types_of_delivery_services_needed[]"]:checked').length;
                    const pickupDays = document.getElementById('preferred_pickup_days').selectedOptions.length;
                    const pickupFromTime = document.querySelector('input[name="preferred_pickup_from_time"]').value;
                    const pickupToTime = document.querySelector('input[name="preferred_pickup_to_time"]').value;

                    if (deliveryServices === 0 || pickupDays === 0 || !pickupFromTime || !pickupToTime) {
                        Swal.fire({
                            position: 'center',
                            icon: 'error',
                            title: 'Error',
                            text: 'Please fill all required fields',
                            showConfirmButton: false,
                            showCloseButton: true,
                            allowOutsideClick: false,
                            allowEscapeKey: false,
                            backdrop: true,
                        });
                        isValid = false;
                    }
                } else if (step === 4) {
                    const logisticsProvider = document.querySelector('input[name="logistics_provider"]:checked');
                    const paymentMethods = document.querySelectorAll(
                        'input[name="preferred_payment_method[]"]:checked').length;
                    const billingContactName = document.getElementById('billing_contact_name').value;
                    const billingEmailAddress = document.getElementById('billing_email_address').value;
                    const taxId = document.getElementById('tax_id').value;

                    if (!logisticsProvider || paymentMethods === 0 || !billingContactName || !billingEmailAddress ||
                        !taxId) {
                        Swal.fire({
                            position: 'center',
                            icon: 'error',
                            title: 'Error',
                            text: 'Please fill all required fields',
                            showConfirmButton: false,
                            showCloseButton: true,
                            allowOutsideClick: false,
                            allowEscapeKey: false,
                            backdrop: true,
                        });
                        isValid = false;
                    }
                } else if (step === 5) {
                    const shipmentTracking = document.querySelector('input[name="shipment_tracking"]:checked');
                    const pickupMaterials = document.querySelector(
                        'input[name="pickup_materials_supplied"]:checked');
                    const demoCall = document.querySelector('input[name="demo_or_onboarding_call"]:checked');
                    const communicationMethods = document.querySelectorAll(
                        'input[name="preferred_communication_method[]"]:checked').length;

                    if (!shipmentTracking || !pickupMaterials || !demoCall || communicationMethods === 0) {
                        Swal.fire({
                            position: 'center',
                            icon: 'error',
                            title: 'Error',
                            text: 'Please fill all required fields',
                            showConfirmButton: false,
                            showCloseButton: true,
                            allowOutsideClick: false,
                            allowEscapeKey: false,
                            backdrop: true,
                        });
                        isValid = false;
                    }
                } else if (step === 6) {
                    const specialNotes = document.querySelector('textarea[name="special_notes"]').value;
                    if (!specialNotes) {
                        Swal.fire({
                            position: 'center',
                            icon: 'error',
                            title: 'Error',
                            text: 'Please provide any special instructions or notes',
                            showConfirmButton: false,
                            showCloseButton: true,
                            allowOutsideClick: false,
                            allowEscapeKey: false,
                            backdrop: true,
                        });
                        isValid = false;
                    }
                }

                return isValid;
            }


            // File upload name display
            document.getElementById('user_image').addEventListener('change', function() {
                const fileName = this.files[0]?.name || 'No file chosen';
                document.getElementById('user_image_name').textContent = fileName;
            });
        });
    </script>
@endsection
