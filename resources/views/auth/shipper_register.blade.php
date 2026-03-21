@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 900px; width: 100%;">

        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 30%;">
        </div>

        <div class="mb-4">
            <h4 class="text-center col-blue mb-1">Shipper Account Registration</h4>
            <p class="text-muted text-center mb-0">Complete all steps to create your Shipper account</p>
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
                <div class="step-label active" data-step="1">Account</div>
                <div class="step-label" data-step="2">Personal</div>
                <div class="step-label" data-step="3">Company</div>
                <div class="step-label" data-step="4">Documents</div>
                <div class="step-label" data-step="5">Compliance</div>
            </div>
        </div>

        <form action="{{ route('register.shipper') }}" method="POST" enctype="multipart/form-data" id="shipperRegForm">
            @csrf
            <input type="hidden" name="role_id" value="{{ $id }}">

            {{-- ─── STEP 1: Account Information ────────────────────────────── --}}
            <div class="step-content active" data-step="1">
                <h5 class="mb-3">Account Information</h5>
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
                        <input type="text" class="form-control" name="phone_no"
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
                        <small id="password-error" class="text-danger d-none">
                            Password must be at least 8 characters and contain a letter and number.
                        </small>
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
                </div>
            </div>

            {{-- ─── STEP 2: Personal Information ───────────────────────────── --}}
            <div class="step-content" data-step="2">
                <h5 class="mb-3">Personal Information</h5>
                <hr>
                <div class="row">
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
                        <select class="form-control" name="gender" id="gender" required>
                            <option disabled {{ old('gender') ? '' : 'selected' }}>Select Gender</option>
                            <option value="Male" {{ old('gender') == 'Male' ? 'selected' : '' }}>Male</option>
                            <option value="Female" {{ old('gender') == 'Female' ? 'selected' : '' }}>Female</option>
                            <option value="Other" {{ old('gender') == 'Other' ? 'selected' : '' }}>Other</option>
                        </select>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Nationality <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="nationality"
                            placeholder="e.g. American" value="{{ old('nationality') }}" required>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 3: Company Information ────────────────────────────── --}}
            <div class="step-content" data-step="3">
                <h5 class="mb-3">Company Information</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Company Name <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="company_name"
                            placeholder="Legal company name" value="{{ old('company_name') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Registration Number <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="registration_number"
                            placeholder="Company registration #" value="{{ old('registration_number') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Tax ID (EIN) <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="tax_id"
                            placeholder="e.g. 12-3456789" value="{{ old('tax_id') }}" required>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Country of Incorporation <span class="text-danger">*</span></label>
                        <input type="text" class="form-control" name="country_of_incorporation"
                            placeholder="e.g. United States" value="{{ old('country_of_incorporation') }}" required>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 4: Upload Documents ────────────────────────────────── --}}
            <div class="step-content" data-step="4">
                <h5 class="mb-3">Upload Documents</h5>
                <hr>
                <div class="row">
                    <div class="col-md-6 mb-3">
                        <label>Government ID <span class="text-danger">*</span></label>
                        <input type="file" class="form-control" name="government_id" id="government_id"
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
                        <label>Certificate of Incorporation</label>
                        <input type="file" class="form-control" name="certificate_of_incorporation"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                    <div class="col-md-6 mb-3">
                        <label>Tax Registration Certificate</label>
                        <input type="file" class="form-control" name="tax_registration_certificate"
                            accept=".jpeg,.jpg,.png,.pdf">
                        <small class="text-muted">Optional · JPEG, PNG or PDF · max 5 MB</small>
                    </div>
                </div>
            </div>

            {{-- ─── STEP 5: Compliance ──────────────────────────────────────── --}}
            <div class="step-content" data-step="5">
                <h5 class="mb-3">Compliance</h5>
                <hr>

                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" name="consent_sanctions_screening"
                        id="consent_sanctions_screening" value="1"
                        {{ old('consent_sanctions_screening') ? 'checked' : '' }} required>
                    <label class="form-check-label" for="consent_sanctions_screening">
                        I consent to sanctions screening <span class="text-danger">*</span>
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

                <div class="mb-3">
                    <label for="source_of_funds">Source of Funds <span class="text-danger">*</span></label>
                    <textarea class="form-control" name="source_of_funds" id="source_of_funds" rows="3"
                        placeholder="Describe the source of your business funds" required>{{ old('source_of_funds') }}</textarea>
                </div>

                <div class="form-check mb-3">
                    <input class="form-check-input" type="checkbox" name="agree_aml_policies"
                        id="agree_aml_policies" value="1"
                        {{ old('agree_aml_policies') ? 'checked' : '' }} required>
                    <label class="form-check-label" for="agree_aml_policies">
                        I agree to the AML (Anti-Money Laundering) policies <span class="text-danger">*</span>
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

            {{-- Error / success messages --}}
            @if ($errors->any())
                <div class="col-12 text-danger text-center mt-3">
                    {{ $errors->first() }}
                </div>
            @endif
            @if (session('success'))
                <div class="col-12 text-success text-center mt-3">
                    {{ session('success') }}
                </div>
            @endif

            <p class="text-center text-muted mt-3">
                Already have an account?
                <a href="{{ route('normal-login', ['id' => $id]) }}">Sign In</a>
            </p>
        </form>
    </div>

    <script>
        document.addEventListener('DOMContentLoaded', function () {

            const steps        = document.querySelectorAll('.step-content');
            const stepDots     = document.querySelectorAll('.numbering-wizard .step');
            const stepLabels   = document.querySelectorAll('.step-label');
            const prevBtn      = document.getElementById('prevBtn');
            const nextBtn      = document.getElementById('nextBtn');
            const submitBtn    = document.getElementById('submitBtn');
            let currentStep    = 1;
            const totalSteps   = steps.length;

            updateProgressBar();

            nextBtn.addEventListener('click', function () {
                if (!validateStep(currentStep)) return;
                if (currentStep < totalSteps) {
                    goTo(currentStep + 1);
                }
            });

            prevBtn.addEventListener('click', function () {
                if (currentStep > 1) {
                    goTo(currentStep - 1);
                }
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
                prevBtn.disabled              = currentStep === 1;
                nextBtn.style.display         = currentStep === totalSteps ? 'none'  : 'inline-block';
                submitBtn.style.display       = currentStep === totalSteps ? 'inline-block' : 'none';
            }

            function updateProgressBar() {
                const bar = document.querySelector('.numbering-wizard .progress-bar');
                const pct = ((currentStep - 1) / (totalSteps - 1)) * 100;
                bar.style.background =
                    `linear-gradient(to right, #1F537B ${pct}%, #e9ecef ${pct}%)`;
            }

            function validateStep(step) {
                let ok = true;
                let msg = '';

                if (step === 1) {
                    const email   = document.getElementById('email').value.trim();
                    const phone   = document.querySelector('[name="phone_no"]').value.trim();
                    const pass    = document.getElementById('password').value;
                    const confirm = document.getElementById('confirm-password').value;
                    if (!email || !phone || !pass || !confirm) {
                        msg = 'Please fill in all account fields.';
                        ok = false;
                    } else if (pass !== confirm) {
                        msg = 'Passwords do not match.';
                        ok = false;
                    } else if (pass.length < 8) {
                        msg = 'Password must be at least 8 characters.';
                        ok = false;
                    }
                } else if (step === 2) {
                    const name        = document.getElementById('name').value.trim();
                    const dob         = document.querySelector('[name="dob"]').value;
                    const gender      = document.getElementById('gender').value;
                    const nationality = document.querySelector('[name="nationality"]').value.trim();
                    if (!name || !dob || !gender || gender === 'Select Gender' || !nationality) {
                        msg = 'Please fill in all personal information fields.';
                        ok = false;
                    }
                } else if (step === 3) {
                    const company   = document.querySelector('[name="company_name"]').value.trim();
                    const regNo     = document.querySelector('[name="registration_number"]').value.trim();
                    const taxId     = document.querySelector('[name="tax_id"]').value.trim();
                    const country   = document.querySelector('[name="country_of_incorporation"]').value.trim();
                    if (!company || !regNo || !taxId || !country) {
                        msg = 'Please fill in all company information fields.';
                        ok = false;
                    }
                } else if (step === 4) {
                    const govId  = document.getElementById('government_id').files.length;
                    const selfie = document.getElementById('selfie').files.length;
                    if (!govId || !selfie) {
                        msg = 'Government ID and Selfie are required.';
                        ok = false;
                    }
                } else if (step === 5) {
                    const consent = document.getElementById('consent_sanctions_screening').checked;
                    const aml     = document.getElementById('agree_aml_policies').checked;
                    const funds   = document.getElementById('source_of_funds').value.trim();
                    if (!consent || !aml || !funds) {
                        msg = 'Please complete all compliance fields and accept required declarations.';
                        ok = false;
                    }
                }

                if (!ok) {
                    Swal.fire({
                        position: 'center',
                        icon: 'error',
                        title: 'Required Fields',
                        text: msg,
                        showConfirmButton: false,
                        showCloseButton: true,
                        allowOutsideClick: false,
                    });
                }
                return ok;
            }
        });
    </script>
@endsection
