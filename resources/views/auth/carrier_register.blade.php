@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 960px; width: 100%;">

        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 30%;">
        </div>

        <div class="mb-4">
            <h4 class="text-center col-blue mb-1">Carrier Account Registration</h4>
            <p class="text-muted text-center mb-0">Complete all steps to create your Carrier account</p>
        </div>

        {{-- Step progress wizard --}}
        <div class="numbering-wizard mb-4">
            <div class="d-flex justify-content-between position-relative">
                <div class="step active" data-step="1">1</div>
                <div class="step" data-step="2">2</div>
                <div class="step" data-step="3">3</div>
                <div class="step" data-step="4">4</div>
                <div class="step" data-step="5">5</div>
                <div class="step" data-step="6">6</div>
                <div class="step" data-step="7">7</div>
                <div class="progress-bar position-absolute"></div>
            </div>
            <div class="d-flex justify-content-between mt-2">
                <div class="step-label active" data-step="1">Identity</div>
                <div class="step-label" data-step="2">Documents</div>
                <div class="step-label" data-step="3">License</div>
                <div class="step-label" data-step="4">Regulatory</div>
                <div class="step-label" data-step="5">Insurance</div>
                <div class="step-label" data-step="6">Vehicle</div>
                <div class="step-label" data-step="7">Company</div>
            </div>
        </div>

        <form action="{{ route('register.carrier') }}" method="POST" enctype="multipart/form-data" id="carrierRegForm">
            @csrf
            <input type="hidden" name="role_id" value="{{ $id }}">

            {{-- ─── STEP 1: Account & Identity ─────────────────────────────── --}}
            <div class="step-content active" data-step="1">
                <h5 class="mb-3">Account &amp; Identity Verification</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3 position-relative">
                        <label>Email Address <span class="text-danger">*</span></label>
                        <div class="input-group">
                            <input id="email" class="form-control pe-5 rounded-2" type="email" name="email"
                                placeholder="you@example.com" value="{{ old('email') }}" required>
                            <i id="email-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Phone Number <span class="text-danger">*</span></label>
                        <input type="tel" class="form-control" name="phone_no"
                            placeholder="+1 (000) 000-0000" value="{{ old('phone_no') }}" required>
                    </div>
                    <div class="col-md-6 mb-3 position-relative">
                        <label>Password <span class="text-danger">*</span></label>
                        <div class="input-group">
                            <input id="password" class="form-control pe-5 rounded-2" type="password"
                                name="password" placeholder="Min. 8 characters" required>
                            <i id="password-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                        <small id="password-error" class="text-danger d-none">Password must be at least 8 characters.</small>
                    </div>
                    <div class="col-md-6 mb-3 position-relative">
                        <label>Confirm Password <span class="text-danger">*</span></label>
                        <div class="input-group">
                            <input id="confirm-password" class="form-control pe-5 rounded-2" type="password"
                                name="password_confirmation" placeholder="Repeat password" required>
                            <i id="confirm-password-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                    </div>
                    <div class="col-md-6 mb-3 position-relative">
                        <label>Full Legal Name <span class="text-danger">*</span></label>
                        <div class="input-group">
                            <input id="name" class="form-control pe-5 rounded-2" type="text" name="name"
                                placeholder="As on government ID" value="{{ old('name') }}" required>
                            <i id="name-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Date of Birth <span class="text-danger">*</span></label>
                        <input type="date" class="form-control" name="dob"
                            value="{{ old('dob') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Nationality <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="nationality"
                            placeholder="e.g. American" value="{{ old('nationality') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Government ID Number <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="gov_id_number"
                            placeholder="Passport / National ID number" value="{{ old('gov_id_number') }}" required>
                    </div>
                    <div class="col-md-12 mb-3">
                        <label>Residential Address <span class="text-danger">*</span></label>
                        <textarea class="form-control" name="address" rows="2"
                            placeholder="Full residential address" required>{{ old('address') }}</textarea>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 2: KYC Document Uploads ───────────────────────────── --}}
            <div class="step-content" data-step="2">
                <h5 class="mb-3">KYC Document Uploads</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Government ID — Front <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="gov_id_front" id="gov_id_front"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Government ID — Back <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="gov_id_back" id="gov_id_back"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Selfie / Facial Verification <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="selfie" id="selfie"
                            accept=".jpeg,.jpg,.png" required>
                        <small class="text-muted">JPEG or PNG · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Proof of Address</label>
                        <input type="file" class="form-control" name="proof_address"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 3: Driver & Transport Licensing ────────────────────── --}}
            <div class="step-content" data-step="3">
                <h5 class="mb-3">Driver &amp; Transport Licensing</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>CDL Number <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="cdl_number"
                            placeholder="Commercial Driver License #" value="{{ old('cdl_number') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>CDL Expiry Date <span class="text-danger">*</span></label>
                        <input type="date" class="form-control" name="cdl_expiry"
                            value="{{ old('cdl_expiry') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>CDL Class <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="cdl_class"
                            placeholder="e.g. Class A, Class B" value="{{ old('cdl_class') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>CDL Upload <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="cdl_upload" id="cdl_upload"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Driving Record</label>
                        <input type="file" class="form-control" name="driving_record"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 4: Regulatory Transport Registration ───────────────── --}}
            <div class="step-content" data-step="4">
                <h5 class="mb-3">Regulatory Transport Registration</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Country <span class="text-danger">*</span></label>
                        <select class="form-control" name="regulatory_country" id="regulatory_country" required>
                            <option disabled {{ old('regulatory_country') ? '' : 'selected' }}>Select Country</option>
                            <option value="USA" {{ old('regulatory_country') == 'USA' ? 'selected' : '' }}>USA</option>
                            <option value="Pakistan" {{ old('regulatory_country') == 'Pakistan' ? 'selected' : '' }}>Pakistan</option>
                            <option value="EU" {{ old('regulatory_country') == 'EU' ? 'selected' : '' }}>EU</option>
                        </select>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>USDOT Number <small class="text-muted">(USA)</small></label>
                        <input type="text" class="form-control" name="usdot_number"
                            placeholder="US Department of Transportation #" value="{{ old('usdot_number') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>MC Number <small class="text-muted">(USA)</small></label>
                        <input type="text" class="form-control" name="mc_number"
                            placeholder="Motor Carrier #" value="{{ old('mc_number') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>NTN <small class="text-muted">(Pakistan)</small></label>
                        <input type="text" class="form-control" name="ntn"
                            placeholder="National Tax Number" value="{{ old('ntn') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>VAT Number <small class="text-muted">(EU)</small></label>
                        <input type="text" class="form-control" name="vat_number"
                            placeholder="Value Added Tax registration #" value="{{ old('vat_number') }}">
                    </div>
                </div>
            </div>

            {{-- ─── STEP 5: Insurance ───────────────────────────────────────── --}}
            <div class="step-content" data-step="5">
                <h5 class="mb-3">Insurance</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Auto Insurance Certificate <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="auto_insurance" id="auto_insurance"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Cargo Insurance Certificate <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="cargo_insurance" id="cargo_insurance"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Insurance Expiry Date <span class="text-danger">*</span></label>
                        <input type="date" class="form-control" name="insurance_expiry"
                            value="{{ old('insurance_expiry') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Coverage Limits <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="coverage_limits"
                            placeholder="e.g. $1,000,000 per occurrence" value="{{ old('coverage_limits') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Insurer Name <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="insurer_name"
                            placeholder="Insurance company name" value="{{ old('insurer_name') }}" required>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 6: Vehicle Information ─────────────────────────────── --}}
            <div class="step-content" data-step="6">
                <h5 class="mb-3">Vehicle Information</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Vehicle Registration Number <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="vehicle_reg"
                            placeholder="License plate / reg #" value="{{ old('vehicle_reg') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Make / Model <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="vehicle_make_model"
                            placeholder="e.g. Freightliner Cascadia" value="{{ old('vehicle_make_model') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Year <span class="text-danger">*</span></label>
                        <input type="number" class="form-control" name="vehicle_year"
                            placeholder="e.g. 2022" min="1990" max="2030"
                            value="{{ old('vehicle_year') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Vehicle Type <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="vehicle_type"
                            placeholder="e.g. Semi-truck, Box truck, Flatbed"
                            value="{{ old('vehicle_type') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Load Capacity <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="load_capacity"
                            placeholder="e.g. 40,000 lbs" value="{{ old('load_capacity') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Vehicle Registration Document <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="vehicle_doc" id="vehicle_doc"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 7: Company & Compliance ───────────────────────────── --}}
            <div class="step-content" data-step="7">
                <h5 class="mb-3">Company Information</h5>
                <hr>
                <div class="row mb-2">
                    <div class="col-md-6 mb-3">
                        <label>Company Name</label>
                        <input type="text" class="form-control" name="company_name"
                            placeholder="Legal company name (if applicable)" value="{{ old('company_name') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Business Registration Number</label>
                        <input type="text" class="form-control" name="registration_number"
                            placeholder="LLC / Corp reg #" value="{{ old('registration_number') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Tax ID (EIN)</label>
                        <input type="text" class="form-control" name="tax_id"
                            placeholder="e.g. 12-3456789" value="{{ old('tax_id') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Country of Incorporation</label>
                        <input type="text" class="form-control" name="country_of_incorporation"
                            placeholder="e.g. United States" value="{{ old('country_of_incorporation') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Registered Company Address</label>
                        <textarea class="form-control" name="company_address" rows="2"
                            placeholder="Company registered address">{{ old('company_address') }}</textarea>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Certificate of Incorporation</label>
                        <input type="file" class="form-control" name="incorporation"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Bank Account Verification</label>
                        <input type="text" class="form-control" name="bank_account"
                            placeholder="Bank account number or IBAN" value="{{ old('bank_account') }}">
                    </div>
                </div>

                <h6 class="mt-2 mb-3">Risk &amp; Compliance Declarations</h6>
                <hr>

                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" name="criminal_declaration"
                        id="criminal_declaration" value="1"
                        {{ old('criminal_declaration') ? 'checked' : '' }}>
                    <label class="form-check-label" for="criminal_declaration">
                        I declare that I have no criminal history related to transport or fraud
                    </label>
                </div>

                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" name="politically_exposed_person"
                        id="politically_exposed_person" value="1"
                        {{ old('politically_exposed_person') ? 'checked' : '' }}>
                    <label class="form-check-label" for="politically_exposed_person">
                        I declare that I am a Politically Exposed Person (PEP)
                    </label>
                </div>

                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" name="consent_sanctions_screening"
                        id="consent_sanctions_screening" value="1"
                        {{ old('consent_sanctions_screening') ? 'checked' : '' }} required>
                    <label class="form-check-label" for="consent_sanctions_screening">
                        I consent to sanctions screening <span class="text-danger">*</span>
                    </label>
                </div>

                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" name="terms_agreed"
                        id="terms_agreed" value="1"
                        {{ old('terms_agreed') ? 'checked' : '' }} required>
                    <label class="form-check-label" for="terms_agreed">
                        I agree to the Terms &amp; Conditions and Liability Waiver
                        <span class="text-danger">*</span>
                    </label>
                </div>
            </div>

            {{-- Navigation buttons --}}
            <div class="d-flex justify-content-between mt-4">
                <button type="button" class="btn btn-secondary" id="prevBtn" disabled>Previous</button>
                <div>
                    <button type="button" class="btn btn-primary" id="nextBtn">Next</button>
                    <button type="submit" class="btn btn-success" id="submitBtn" style="display:none;">
                        Submit &amp; Send OTP
                    </button>
                </div>
            </div>

            @if ($errors->any())
                <div class="col-12 text-danger text-center mt-3">{{ $errors->first() }}</div>
            @endif
            @if (session('success'))
                <div class="col-12 text-success text-center mt-3">{{ session('success') }}</div>
            @endif

            <p class="text-center text-muted mt-3">
                Already have an account?
                <a href="{{ route('normal-login', ['id' => $id]) }}">Sign In</a>
            </p>
        </form>
    </div>

    <script>
        document.addEventListener('DOMContentLoaded', function () {

            const steps      = document.querySelectorAll('.step-content');
            const stepDots   = document.querySelectorAll('.numbering-wizard .step');
            const stepLabels = document.querySelectorAll('.step-label');
            const prevBtn    = document.getElementById('prevBtn');
            const nextBtn    = document.getElementById('nextBtn');
            const submitBtn  = document.getElementById('submitBtn');
            let currentStep  = 1;
            const totalSteps = steps.length; // 7

            updateProgressBar();

            nextBtn.addEventListener('click', function () {
                if (!validateStep(currentStep)) return;
                if (currentStep < totalSteps) goTo(currentStep + 1);
            });

            prevBtn.addEventListener('click', function () {
                if (currentStep > 1) goTo(currentStep - 1);
            });

            function goTo(n) {
                document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.remove('active');
                document.querySelector(`.numbering-wizard .step[data-step="${currentStep}"]`).classList.remove('active');
                document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.remove('active');

                currentStep = n;

                document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.add('active');
                document.querySelector(`.numbering-wizard .step[data-step="${currentStep}"]`).classList.add('active');
                document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.add('active');

                updateButtons();
                updateProgressBar();
            }

            function updateButtons() {
                prevBtn.disabled        = currentStep === 1;
                nextBtn.style.display   = currentStep === totalSteps ? 'none'         : 'inline-block';
                submitBtn.style.display = currentStep === totalSteps ? 'inline-block' : 'none';
            }

            function updateProgressBar() {
                const bar = document.querySelector('.numbering-wizard .progress-bar');
                const pct = ((currentStep - 1) / (totalSteps - 1)) * 100;
                bar.style.background =
                    `linear-gradient(to right, #1F537B ${pct}%, #e9ecef ${pct}%)`;
            }

            function validateStep(step) {
                let msg = '';

                if (step === 1) {
                    const email   = document.getElementById('email').value.trim();
                    const phone   = document.querySelector('[name="phone_no"]').value.trim();
                    const pass    = document.getElementById('password').value;
                    const confirm = document.getElementById('confirm-password').value;
                    const name    = document.getElementById('name').value.trim();
                    const dob     = document.querySelector('[name="dob"]').value;
                    const nat     = document.querySelector('[name="nationality"]').value.trim();
                    const govId   = document.querySelector('[name="gov_id_number"]').value.trim();
                    const addr    = document.querySelector('[name="address"]').value.trim();

                    if (!email || !phone || !pass || !confirm || !name || !dob || !nat || !govId || !addr) {
                        msg = 'Please fill in all required fields.';
                    } else if (pass !== confirm) {
                        msg = 'Passwords do not match.';
                    } else if (pass.length < 8) {
                        msg = 'Password must be at least 8 characters.';
                    }

                } else if (step === 2) {
                    const front  = document.getElementById('gov_id_front').files.length;
                    const back   = document.getElementById('gov_id_back').files.length;
                    const selfie = document.getElementById('selfie').files.length;
                    if (!front || !back || !selfie) {
                        msg = 'Government ID (front & back) and Selfie are required.';
                    }

                } else if (step === 3) {
                    const cdlNo     = document.querySelector('[name="cdl_number"]').value.trim();
                    const cdlExpiry = document.querySelector('[name="cdl_expiry"]').value;
                    const cdlClass  = document.querySelector('[name="cdl_class"]').value.trim();
                    const cdlFile   = document.getElementById('cdl_upload').files.length;
                    if (!cdlNo || !cdlExpiry || !cdlClass || !cdlFile) {
                        msg = 'CDL number, expiry, class and upload are required.';
                    }

                } else if (step === 4) {
                    const country = document.getElementById('regulatory_country').value;
                    if (!country || country === 'Select Country') {
                        msg = 'Please select your country.';
                    }

                } else if (step === 5) {
                    const autoIns    = document.getElementById('auto_insurance').files.length;
                    const cargoIns   = document.getElementById('cargo_insurance').files.length;
                    const expiry     = document.querySelector('[name="insurance_expiry"]').value;
                    const coverage   = document.querySelector('[name="coverage_limits"]').value.trim();
                    const insurer    = document.querySelector('[name="insurer_name"]').value.trim();
                    if (!autoIns || !cargoIns || !expiry || !coverage || !insurer) {
                        msg = 'All insurance fields and documents are required.';
                    }

                } else if (step === 6) {
                    const reg     = document.querySelector('[name="vehicle_reg"]').value.trim();
                    const model   = document.querySelector('[name="vehicle_make_model"]').value.trim();
                    const year    = document.querySelector('[name="vehicle_year"]').value;
                    const type    = document.querySelector('[name="vehicle_type"]').value.trim();
                    const cap     = document.querySelector('[name="load_capacity"]').value.trim();
                    const vDoc    = document.getElementById('vehicle_doc').files.length;
                    if (!reg || !model || !year || !type || !cap || !vDoc) {
                        msg = 'All vehicle fields and the registration document are required.';
                    }

                } else if (step === 7) {
                    const sanctions = document.getElementById('consent_sanctions_screening').checked;
                    const terms     = document.getElementById('terms_agreed').checked;
                    if (!sanctions || !terms) {
                        msg = 'You must consent to sanctions screening and agree to the Terms & Liability Waiver.';
                    }
                }

                if (msg) {
                    Swal.fire({
                        position: 'center',
                        icon: 'error',
                        title: 'Required Fields',
                        text: msg,
                        showConfirmButton: false,
                        showCloseButton: true,
                        allowOutsideClick: false,
                    });
                    return false;
                }
                return true;
            }
        });
    </script>
@endsection
