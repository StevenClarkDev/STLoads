@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 960px; width: 100%;">

        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 30%;">
        </div>

        <div class="mb-4">
            <h4 class="text-center col-blue mb-1">Broker Account Registration</h4>
            <p class="text-muted text-center mb-0">Complete all steps to create your Broker account</p>
        </div>

        {{-- Step progress wizard --}}
        <div class="numbering-wizard mb-4">
            <div class="d-flex justify-content-between position-relative">
                <div class="step active" data-step="1">1</div>
                <div class="step" data-step="2">2</div>
                <div class="step" data-step="3">3</div>
                <div class="step" data-step="4">4</div>
                <div class="step" data-step="5">5</div>
                <div class="progress-bar position-absolute"></div>
            </div>
            <div class="d-flex justify-content-between mt-2">
                <div class="step-label active" data-step="1">Identity</div>
                <div class="step-label" data-step="2">Documents</div>
                <div class="step-label" data-step="3">Brokerage</div>
                <div class="step-label" data-step="4">Company</div>
                <div class="step-label" data-step="5">Compliance</div>
            </div>
        </div>

        @php
            $errorStep = 1;
            if ($errors->hasAny(['gov_id_front', 'gov_id_back', 'selfie', 'proof_address'])) {
                $errorStep = 2;
            } elseif ($errors->hasAny(['fmcsa_broker_license', 'mc_authority_number', 'surety_bond_number'])) {
                $errorStep = 3;
            } elseif ($errors->hasAny(['company_name', 'registration_number', 'tax_id', 'country_of_incorporation', 'company_address', 'bank_account'])) {
                $errorStep = 4;
            } elseif ($errors->hasAny(['consent_sanctions_screening', 'source_of_funds', 'agree_aml_policies'])) {
                $errorStep = 5;
            }
        @endphp

        <form action="{{ route('register.broker') }}" method="POST" enctype="multipart/form-data" id="brokerRegForm">
            @csrf
            <input type="hidden" name="role_id" value="{{ $id }}">

            @if ($errors->any())
                <div class="alert alert-danger alert-dismissible fade show mb-4" role="alert">
                    <strong><i class="fas fa-exclamation-triangle me-1"></i> Please fix the following error:</strong>
                    {{ $errors->first() }}
                    <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>
            @endif

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
                                class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3"
                                style="cursor:pointer" title="Show/hide password"></i>
                        </div>
                        <small id="password-error" class="text-danger d-none">Password must be at least 8 characters.</small>
                    </div>
                    <div class="col-md-6 mb-3 position-relative">
                        <label>Confirm Password <span class="text-danger">*</span></label>
                        <div class="input-group">
                            <input id="confirm-password" class="form-control pe-5 rounded-2" type="password"
                                name="password_confirmation" placeholder="Repeat password" required>
                            <i id="confirm-password-icon"
                                class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3"
                                style="cursor:pointer" title="Show/hide password"></i>
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
                        <label>Gender <span class="text-danger">*</span></label>
                        <select class="form-control" name="gender" required>
                            <option disabled {{ old('gender') ? '' : 'selected' }}>Select Gender</option>
                            <option value="Male" {{ old('gender') === 'Male' ? 'selected' : '' }}>Male</option>
                            <option value="Female" {{ old('gender') === 'Female' ? 'selected' : '' }}>Female</option>
                            <option value="Other" {{ old('gender') === 'Other' ? 'selected' : '' }}>Other</option>
                        </select>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Nationality <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="nationality"
                            placeholder="e.g. American" value="{{ old('nationality') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>SSN # <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="ssn_no"
                            placeholder="000-00-0000" value="{{ old('ssn_no') }}" required
                            pattern="\d{3}-?\d{2}-?\d{4}" maxlength="11">
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

            {{-- ─── STEP 3: Brokerage Credentials ──────────────────────────── --}}
            <div class="step-content" data-step="3">
                <h5 class="mb-3">Brokerage Licensing &amp; Credentials</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>FMCSA Broker License # <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="fmcsa_broker_license"
                            placeholder="MC-XXXXXX" value="{{ old('fmcsa_broker_license') }}" required>
                        <small class="text-muted">Federal Motor Carrier Safety Administration broker license number</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>MC Authority Number <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="mc_authority_number"
                            placeholder="MC-XXXXXX" value="{{ old('mc_authority_number') }}" required>
                        <small class="text-muted">Motor Carrier authority number issued by FMCSA</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Surety Bond Number</label>
                        <input type="text" class="form-control" name="surety_bond_number"
                            placeholder="BMC-84 Surety Bond #" value="{{ old('surety_bond_number') }}">
                        <small class="text-muted">Optional · Required for FMCSA compliance</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>FMCSA License Document</label>
                        <input type="file" class="form-control" name="fmcsa_license_doc" id="fmcsa_license_doc"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Surety Bond Document</label>
                        <input type="file" class="form-control" name="surety_bond_doc"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 4: Company Information ─────────────────────────────── --}}
            <div class="step-content" data-step="4">
                <h5 class="mb-3">Company Information</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Company / Brokerage Name <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="company_name"
                            placeholder="Legal company name" value="{{ old('company_name') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Registration Number <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="registration_number"
                            placeholder="State / Federal registration #" value="{{ old('registration_number') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Tax ID (EIN) <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="tax_id"
                            placeholder="XX-XXXXXXX" value="{{ old('tax_id') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Country of Incorporation <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="country_of_incorporation"
                            placeholder="e.g. United States" value="{{ old('country_of_incorporation') }}" required>
                    </div>
                    <div class="col-md-12 mb-3">
                        <label>Company Address <span class="text-danger">*</span></label>
                        <textarea class="form-control" name="company_address" rows="2"
                            placeholder="Principal place of business" required>{{ old('company_address') }}</textarea>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Bank Account Number</label>
                        <input type="text" class="form-control" name="bank_account"
                            placeholder="For payment processing" value="{{ old('bank_account') }}">
                        <small class="text-muted">Optional</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Certificate of Incorporation</label>
                        <input type="file" class="form-control" name="incorporation_doc"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 5: Compliance ──────────────────────────────────────── --}}
            <div class="step-content" data-step="5">
                <h5 class="mb-3">Compliance &amp; Declarations</h5>
                <hr>
                <div class="row">
                    <div class="col-12 mb-3">
                        <label>Source of Funds <span class="text-danger">*</span></label>
                        <textarea class="form-control" name="source_of_funds" rows="3"
                            placeholder="Describe the primary source of your business funds" required>{{ old('source_of_funds') }}</textarea>
                    </div>
                    <div class="col-md-6 mb-3">
                        <div class="form-check">
                            <input class="form-check-input" type="checkbox" name="politically_exposed_person"
                                id="pep" value="1" {{ old('politically_exposed_person') ? 'checked' : '' }}>
                            <label class="form-check-label" for="pep">
                                I am a Politically Exposed Person (PEP)
                            </label>
                        </div>
                    </div>
                    <div class="col-12 mb-3">
                        <div class="form-check">
                            <input class="form-check-input" type="checkbox" name="consent_sanctions_screening"
                                id="consent_sanctions_screening" value="1" required>
                            <label class="form-check-label" for="consent_sanctions_screening">
                                I consent to OFAC/sanctions screening. <span class="text-danger">*</span>
                            </label>
                        </div>
                    </div>
                    <div class="col-12 mb-3">
                        <div class="form-check">
                            <input class="form-check-input" type="checkbox" name="agree_aml_policies"
                                id="agree_aml_policies" value="1" required>
                            <label class="form-check-label" for="agree_aml_policies">
                                I have read and agree to the AML/KYC policies. <span class="text-danger">*</span>
                            </label>
                        </div>
                    </div>
                    <div class="col-12 mb-3">
                        <div class="form-check">
                            <input class="form-check-input" type="checkbox" name="terms_agreed"
                                id="terms_agreed" value="1" required>
                            <label class="form-check-label" for="terms_agreed">
                                I agree to the Terms &amp; Conditions and Liability Waiver. <span class="text-danger">*</span>
                            </label>
                        </div>
                    </div>
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
            const totalSteps = steps.length; // 5

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
                    const gender  = document.querySelector('[name="gender"]').value;
                    const nat     = document.querySelector('[name="nationality"]').value.trim();
                    const ssn     = document.querySelector('[name="ssn_no"]').value.trim();
                    const govId   = document.querySelector('[name="gov_id_number"]').value.trim();
                    const addr    = document.querySelector('[name="address"]').value.trim();

                    if (!email || !phone || !pass || !confirm || !name || !dob || !gender || !nat || !ssn || !govId || !addr) {
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
                    const fmcsa = document.querySelector('[name="fmcsa_broker_license"]').value.trim();
                    const mc    = document.querySelector('[name="mc_authority_number"]').value.trim();
                    if (!fmcsa || !mc) {
                        msg = 'FMCSA Broker License # and MC Authority Number are required.';
                    }

                } else if (step === 4) {
                    const company  = document.querySelector('[name="company_name"]').value.trim();
                    const regNo    = document.querySelector('[name="registration_number"]').value.trim();
                    const taxId    = document.querySelector('[name="tax_id"]').value.trim();
                    const country  = document.querySelector('[name="country_of_incorporation"]').value.trim();
                    const compAddr = document.querySelector('[name="company_address"]').value.trim();
                    if (!company || !regNo || !taxId || !country || !compAddr) {
                        msg = 'Please fill in all required company fields.';
                    }

                } else if (step === 5) {
                    const sanctions = document.getElementById('consent_sanctions_screening').checked;
                    const aml       = document.getElementById('agree_aml_policies').checked;
                    const terms     = document.getElementById('terms_agreed').checked;
                    const funds     = document.querySelector('[name="source_of_funds"]').value.trim();
                    if (!funds) {
                        msg = 'Source of funds is required.';
                    } else if (!sanctions || !aml || !terms) {
                        msg = 'You must consent to sanctions screening, AML policies, and terms.';
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

            // On page reload after server-side validation failure, jump to the error step
            @if ($errors->any())
            goTo({{ $errorStep }});
            window.scrollTo({ top: 0, behavior: 'smooth' });
            @endif

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
