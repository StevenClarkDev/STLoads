@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 960px; width: 100%;">

        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 30%;">
        </div>

        <div class="mb-4">
            <h4 class="text-center col-blue mb-1">Freight Forwarder Account Registration</h4>
            <p class="text-muted text-center mb-0">Complete all steps to create your Freight Forwarder account</p>
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
                <div class="step-label active" data-step="1">Company</div>
                <div class="step-label" data-step="2">Documents</div>
                <div class="step-label" data-step="3">Ownership</div>
                <div class="step-label" data-step="4">Licensing</div>
                <div class="step-label" data-step="5">Insurance</div>
                <div class="step-label" data-step="6">Operations</div>
                <div class="step-label" data-step="7">Financial</div>
            </div>
        </div>

        <form action="{{ route('register.freight-forwarder') }}" method="POST" enctype="multipart/form-data" id="ffRegForm">
            @csrf
            <input type="hidden" name="role_id" value="{{ $id }}">

            {{-- ─── STEP 1: Account Credentials + Company Information ──────────── --}}
            <div class="step-content active" data-step="1">
                <h5 class="mb-3">Account Credentials</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3 position-relative">
                        <label>Email Address <span class="text-danger">*</span></label>
                        <div class="input-group">
                            <input id="email" class="form-control pe-5 rounded-2" type="email" name="email"
                                placeholder="company@example.com" value="{{ old('email') }}" required>
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
                                class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3" style="cursor:pointer" title="Show/hide password"></i>
                        </div>
                        <small id="password-error" class="text-danger d-none">Password must be at least 8 characters.</small>
                    </div>
                    <div class="col-md-6 mb-3 position-relative">
                        <label>Confirm Password <span class="text-danger">*</span></label>
                        <div class="input-group">
                            <input id="confirm-password" class="form-control pe-5 rounded-2" type="password"
                                name="password_confirmation" placeholder="Repeat password" required>
                            <i id="confirm-password-icon"
                                class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3" style="cursor:pointer" title="Show/hide password"></i>
                        </div>
                    </div>
                </div>

                <h5 class="mb-3 mt-2">Company Information</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Registered Legal Name <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="name"
                            placeholder="Full registered company name" value="{{ old('name') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Trade Name <small class="text-muted">(DBA)</small></label>
                        <input type="text" class="form-control" name="trade_name"
                            placeholder="Trading / DBA name" value="{{ old('trade_name') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Business Registration Number</label>
                        <input type="text" class="form-control" name="registration_number"
                            placeholder="LLC / Corp reg #" value="{{ old('registration_number') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Tax ID</label>
                        <input type="text" class="form-control" name="tax_id"
                            placeholder="EIN / Tax ID" value="{{ old('tax_id') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Country of Incorporation</label>
                        <input type="text" class="form-control" name="country_of_incorporation"
                            placeholder="e.g. United States" value="{{ old('country_of_incorporation') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Date of Incorporation</label>
                        <input type="date" class="form-control" name="incorporation_date"
                            value="{{ old('incorporation_date') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Registered Address</label>
                        <textarea class="form-control" name="company_address" rows="2"
                            placeholder="Registered company address">{{ old('company_address') }}</textarea>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Operating Address</label>
                        <textarea class="form-control" name="address" rows="2"
                            placeholder="Primary operating address">{{ old('address') }}</textarea>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 2: Required Documents ─────────────────────────────────── --}}
            <div class="step-content" data-step="2">
                <h5 class="mb-3">Required Documents</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Certificate of Incorporation <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="incorporation_doc" id="incorporation_doc"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Business License <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="business_license" id="business_license"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Tax Registration Certificate</label>
                        <input type="file" class="form-control" name="tax_certificate"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Proof of Address</label>
                        <input type="file" class="form-control" name="proof_address"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 3: Beneficial Ownership (AML) ─────────────────────────── --}}
            <div class="step-content" data-step="3">
                <h5 class="mb-3">Beneficial Ownership (AML)</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Director Full Name</label>
                        <input type="text" class="form-control" name="director_name"
                            placeholder="Full legal name of director" value="{{ old('director_name') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Director Date of Birth</label>
                        <input type="date" class="form-control" name="director_dob"
                            value="{{ old('director_dob') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Director Government ID</label>
                        <input type="file" class="form-control" name="director_id"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Ultimate Beneficial Owner (UBO) Name</label>
                        <input type="text" class="form-control" name="ubo_name"
                            placeholder="UBO full legal name" value="{{ old('ubo_name') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>UBO Date of Birth</label>
                        <input type="date" class="form-control" name="ubo_dob"
                            value="{{ old('ubo_dob') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>UBO Nationality</label>
                        <input type="text" class="form-control" name="ubo_nationality"
                            placeholder="e.g. American" value="{{ old('ubo_nationality') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>UBO Government ID</label>
                        <input type="file" class="form-control" name="ubo_id"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>UBO Residential Address</label>
                        <textarea class="form-control" name="ubo_address" rows="2"
                            placeholder="UBO full residential address">{{ old('ubo_address') }}</textarea>
                    </div>
                    <div class="col-md-12 mb-2">
                        <div class="form-check">
                            <input class="form-check-input" type="checkbox" name="politically_exposed_person"
                                id="pep" value="1"
                                {{ old('politically_exposed_person') ? 'checked' : '' }}>
                            <label class="form-check-label" for="pep">
                                PEP Declaration — I / a UBO is a Politically Exposed Person
                            </label>
                        </div>
                    </div>
                    <div class="col-md-12 mb-2">
                        <div class="form-check">
                            <input class="form-check-input" type="checkbox" name="consent_sanctions_screening"
                                id="sanctions" value="1"
                                {{ old('consent_sanctions_screening') ? 'checked' : '' }}>
                            <label class="form-check-label" for="sanctions">
                                I consent to sanctions screening
                            </label>
                        </div>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 4: Regulatory & Trade Licensing ────────────────────────── --}}
            <div class="step-content" data-step="4">
                <h5 class="mb-3">Regulatory &amp; Trade Licensing</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Country / Region <span class="text-danger">*</span></label>
                        <select class="form-control" name="regulatory_country" id="regulatory_country" required>
                            <option disabled {{ old('regulatory_country') ? '' : 'selected' }}>Select Region</option>
                            <option value="USA" {{ old('regulatory_country') == 'USA' ? 'selected' : '' }}>USA</option>
                            <option value="EU" {{ old('regulatory_country') == 'EU' ? 'selected' : '' }}>EU</option>
                            <option value="Pakistan" {{ old('regulatory_country') == 'Pakistan' ? 'selected' : '' }}>Pakistan</option>
                            <option value="Other" {{ old('regulatory_country') == 'Other' ? 'selected' : '' }}>Other</option>
                        </select>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>FMC License Number <small class="text-muted">(USA)</small></label>
                        <input type="text" class="form-control" name="fmc_license"
                            placeholder="Federal Maritime Commission #" value="{{ old('fmc_license') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>NVOCC Registration <small class="text-muted">(USA)</small></label>
                        <input type="text" class="form-control" name="nvocc_reg"
                            placeholder="Non-Vessel Operating Common Carrier #" value="{{ old('nvocc_reg') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Surety Bond Number</label>
                        <input type="text" class="form-control" name="surety_bond"
                            placeholder="Bond number" value="{{ old('surety_bond') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Customs Broker License</label>
                        <input type="text" class="form-control" name="customs_broker_license"
                            placeholder="Customs broker license #" value="{{ old('customs_broker_license') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>IATA Accreditation</label>
                        <input type="text" class="form-control" name="iata_accreditation"
                            placeholder="IATA code / accreditation #" value="{{ old('iata_accreditation') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>EORI Number <small class="text-muted">(EU)</small></label>
                        <input type="text" class="form-control" name="eori_number"
                            placeholder="Economic Operators Registration #" value="{{ old('eori_number') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>VAT Number</label>
                        <input type="text" class="form-control" name="vat_number"
                            placeholder="Value Added Tax #" value="{{ old('vat_number') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>SECP Registration <small class="text-muted">(Pakistan)</small></label>
                        <input type="text" class="form-control" name="secp_reg"
                            placeholder="Securities &amp; Exchange Commission #" value="{{ old('secp_reg') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Chamber of Commerce Registration</label>
                        <input type="text" class="form-control" name="chamber_reg"
                            placeholder="Chamber reg #" value="{{ old('chamber_reg') }}">
                    </div>
                </div>
            </div>

            {{-- ─── STEP 5: Insurance ───────────────────────────────────────────── --}}
            <div class="step-content" data-step="5">
                <h5 class="mb-3">Insurance</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Insurance Certificate <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="insurance_cert" id="insurance_cert"
                            accept=".jpeg,.jpg,.png,.pdf" required>
                        <small class="text-muted">JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Policy Number <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="policy_number"
                            placeholder="Insurance policy #" value="{{ old('policy_number') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Coverage Limits <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="coverage_limits"
                            placeholder="e.g. $1,000,000 per occurrence" value="{{ old('coverage_limits') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Expiry Date <span class="text-danger">*</span></label>
                        <input type="date" class="form-control" name="insurance_expiry"
                            value="{{ old('insurance_expiry') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Insurer Contact</label>
                        <input type="text" class="form-control" name="insurer_contact"
                            placeholder="Contact name or number" value="{{ old('insurer_contact') }}">
                    </div>
                </div>
            </div>

            {{-- ─── STEP 6: Operational Information ────────────────────────────── --}}
            <div class="step-content" data-step="6">
                <h5 class="mb-3">Operational Information</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Modes Handled <span class="text-danger">*</span></label>
                        <select class="form-control" name="transport_modes" id="transport_modes" required>
                            <option disabled {{ old('transport_modes') ? '' : 'selected' }}>Select Mode</option>
                            <option value="Air" {{ old('transport_modes') == 'Air' ? 'selected' : '' }}>Air</option>
                            <option value="Sea" {{ old('transport_modes') == 'Sea' ? 'selected' : '' }}>Sea</option>
                            <option value="Road" {{ old('transport_modes') == 'Road' ? 'selected' : '' }}>Road</option>
                            <option value="Rail" {{ old('transport_modes') == 'Rail' ? 'selected' : '' }}>Rail</option>
                        </select>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Countries Served</label>
                        <input type="text" class="form-control" name="countries_served"
                            placeholder="e.g. USA, Pakistan, EU, China" value="{{ old('countries_served') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Customs Brokerage</label>
                        <select class="form-control" name="customs_brokerage">
                            <option value="Yes" {{ old('customs_brokerage') == 'Yes' ? 'selected' : '' }}>Yes</option>
                            <option value="No" {{ old('customs_brokerage', 'No') == 'No' ? 'selected' : '' }}>No</option>
                        </select>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Consolidation Services</label>
                        <select class="form-control" name="consolidation_services">
                            <option value="Yes" {{ old('consolidation_services') == 'Yes' ? 'selected' : '' }}>Yes</option>
                            <option value="No" {{ old('consolidation_services', 'No') == 'No' ? 'selected' : '' }}>No</option>
                        </select>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Warehousing</label>
                        <select class="form-control" name="warehousing">
                            <option value="Yes" {{ old('warehousing') == 'Yes' ? 'selected' : '' }}>Yes</option>
                            <option value="No" {{ old('warehousing', 'No') == 'No' ? 'selected' : '' }}>No</option>
                        </select>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Years in Operation</label>
                        <input type="number" class="form-control" name="years_in_operation"
                            placeholder="e.g. 10" min="0" value="{{ old('years_in_operation') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Annual Shipment Volume</label>
                        <input type="text" class="form-control" name="annual_volume"
                            placeholder="e.g. 500 containers / year" value="{{ old('annual_volume') }}">
                    </div>
                </div>
            </div>

            {{-- ─── STEP 7: Financial & AML Controls ───────────────────────────── --}}
            <div class="step-content" data-step="7">
                <h5 class="mb-3">Financial &amp; AML Controls</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Bank Account Verification</label>
                        <input type="text" class="form-control" name="bank_account"
                            placeholder="Bank account number or IBAN" value="{{ old('bank_account') }}">
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Source of Funds</label>
                        <textarea class="form-control" name="source_of_funds" rows="2"
                            placeholder="Describe the primary source of business funds">{{ old('source_of_funds') }}</textarea>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Expected Monthly Transaction Volume</label>
                        <input type="text" class="form-control" name="monthly_transaction_volume"
                            placeholder="e.g. $500,000" value="{{ old('monthly_transaction_volume') }}">
                    </div>
                </div>

                <h6 class="mt-2 mb-3">Compliance Declarations</h6>
                <hr>

                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" name="ofac_consent"
                        id="ofac_consent" value="1"
                        {{ old('ofac_consent') ? 'checked' : '' }}>
                    <label class="form-check-label" for="ofac_consent">
                        I consent to OFAC / Sanctions Screening
                    </label>
                </div>

                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" name="terms_agreed"
                        id="terms_agreed" value="1"
                        {{ old('terms_agreed') ? 'checked' : '' }} required>
                    <label class="form-check-label" for="terms_agreed">
                        I agree to the Terms &amp; Conditions and AML Policy
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
                    const email     = document.getElementById('email').value.trim();
                    const phone     = document.querySelector('[name="phone_no"]').value.trim();
                    const pass      = document.getElementById('password').value;
                    const confirm   = document.getElementById('confirm-password').value;
                    const legalName = document.querySelector('[name="name"]').value.trim();

                    if (!email || !phone || !pass || !confirm || !legalName) {
                        msg = 'Email, phone, password and registered legal name are required.';
                    } else if (pass !== confirm) {
                        msg = 'Passwords do not match.';
                    } else if (pass.length < 8) {
                        msg = 'Password must be at least 8 characters.';
                    }

                } else if (step === 2) {
                    const incDoc = document.getElementById('incorporation_doc').files.length;
                    const bizLic = document.getElementById('business_license').files.length;
                    if (!incDoc || !bizLic) {
                        msg = 'Certificate of Incorporation and Business License are required.';
                    }

                } else if (step === 3) {
                    // All fields optional — no validation required

                } else if (step === 4) {
                    const region = document.getElementById('regulatory_country').value;
                    if (!region || region === 'Select Region') {
                        msg = 'Please select your country / region.';
                    }

                } else if (step === 5) {
                    const cert     = document.getElementById('insurance_cert').files.length;
                    const policy   = document.querySelector('[name="policy_number"]').value.trim();
                    const coverage = document.querySelector('[name="coverage_limits"]').value.trim();
                    const expiry   = document.querySelector('[name="insurance_expiry"]').value;
                    if (!cert || !policy || !coverage || !expiry) {
                        msg = 'Insurance certificate, policy number, coverage limits and expiry date are required.';
                    }

                } else if (step === 6) {
                    const modes = document.getElementById('transport_modes').value;
                    if (!modes || modes === 'Select Mode') {
                        msg = 'Please select the transport mode(s) handled.';
                    }

                } else if (step === 7) {
                    const terms = document.getElementById('terms_agreed').checked;
                    if (!terms) {
                        msg = 'You must agree to the Terms & Conditions and AML Policy.';
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

            // Password show/hide toggle
            document.querySelectorAll('.pwd-toggle').forEach(function(icon) {
                icon.addEventListener('click', function() {
                    var input = this.parentElement.querySelector('input');
                    if (input) {
                        input.type = input.type === 'password' ? 'text' : 'password';
                        this.classList.toggle('fa-eye');
                        this.classList.toggle('fa-eye-slash');
                    }
                });
            });
        });
    </script>
@endsection
