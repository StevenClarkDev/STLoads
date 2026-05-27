use leptos::{ev, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_query_map},
};

use crate::{api, session::use_auth};
use shared::{OtpPurpose, RegisterRequest};

use crate::pages::auth_helpers::*;
#[component]
pub fn RegisterPage() -> impl IntoView {
    let navigate = use_navigate();
    let query = use_query_map();
    let auth = use_auth();
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let address = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirmation = RwSignal::new(String::new());
    let role = RwSignal::new("shipper".to_string());
    let company_name = RwSignal::new(String::new());
    let company_address = RwSignal::new(String::new());
    let dot_number = RwSignal::new(String::new());
    let mc_number = RwSignal::new(String::new());
    let equipment_types = RwSignal::new(String::new());
    let business_entity_id = RwSignal::new(String::new());
    let facility_address = RwSignal::new(String::new());
    let fulfillment_contact_info = RwSignal::new(String::new());
    let fmcsa_broker_license_no = RwSignal::new(String::new());
    let mc_authority_number = RwSignal::new(String::new());
    let freight_forwarder_license = RwSignal::new(String::new());
    let customs_license = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);
    let register_step = RwSignal::new(0_usize);
    let selected_role_label = Memo::new(move |_| role_label(role.get()));
    let title = Signal::derive(move || format!("{} Signup", selected_role_label.get()));
    let subtitle = Signal::derive(move || {
        format!(
            "Create a {} account with a role-specific signup flow, then continue into OTP verification and onboarding.",
            selected_role_label.get().to_lowercase()
        )
    });

    Effect::new(move |_| {
        query.with(|params| {
            if let Some(role_key) = params.get("role")
                && is_supported_role(role_key.as_str())
            {
                role.set(role_key.clone());
            }
            if email.get_untracked().is_empty()
                && let Some(prefill) = params.get("email")
            {
                email.set(prefill.clone());
            }
        });
    });

    let on_next = move |_| {
        feedback.set(None);
        if let Some(message) = validate_register_step(
            register_step.get(),
            role.get().as_str(),
            &name.get(),
            &email.get(),
            &phone.get(),
            &address.get(),
            &password.get(),
            &password_confirmation.get(),
            &company_name.get(),
            &company_address.get(),
            &dot_number.get(),
            &mc_number.get(),
            &equipment_types.get(),
            &business_entity_id.get(),
            &facility_address.get(),
            &fulfillment_contact_info.get(),
            &fmcsa_broker_license_no.get(),
            &mc_authority_number.get(),
            &freight_forwarder_license.get(),
            &customs_license.get(),
        ) {
            feedback.set(Some(message));
            return;
        }
        register_step.set(1);
    };

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth;
        ev.prevent_default();
        feedback.set(None);

        let payload = RegisterRequest {
            name: name.get().trim().to_string(),
            email: email.get().trim().to_string(),
            password: password.get(),
            password_confirmation: password_confirmation.get(),
            role_key: role.get(),
            phone_no: optional_string(phone.get()),
            address: optional_string(address.get()),
        };

        if let Some(message) = validate_register_step(
            register_step.get(),
            payload.role_key.as_str(),
            &payload.name,
            &payload.email,
            &phone.get(),
            &address.get(),
            &payload.password,
            &payload.password_confirmation,
            &company_name.get(),
            &company_address.get(),
            &dot_number.get(),
            &mc_number.get(),
            &equipment_types.get(),
            &business_entity_id.get(),
            &facility_address.get(),
            &fulfillment_contact_info.get(),
            &fmcsa_broker_license_no.get(),
            &mc_authority_number.get(),
            &freight_forwarder_license.get(),
            &customs_license.get(),
        ) {
            feedback.set(Some(message));
            return;
        }

        let pending_draft = PendingOnboardingDraft {
            role_key: payload.role_key.clone(),
            company_name: optional_string(company_name.get()),
            company_address: optional_string(company_address.get()),
            dot_number: optional_string(dot_number.get()),
            mc_number: optional_string(mc_number.get()),
            equipment_types: optional_string(equipment_types.get()),
            business_entity_id: optional_string(business_entity_id.get()),
            facility_address: optional_string(facility_address.get()),
            fulfillment_contact_info: optional_string(fulfillment_contact_info.get()),
            fmcsa_broker_license_no: optional_string(fmcsa_broker_license_no.get()),
            mc_authority_number: optional_string(mc_authority_number.get()),
            freight_forwarder_license: optional_string(freight_forwarder_license.get()),
            customs_license: optional_string(customs_license.get()),
        };

        is_submitting.set(true);
        spawn_local(async move {
            match api::register(&payload).await {
                Ok(response) => {
                    let message = response.message;
                    if let Some(dev_otp) = response.dev_otp {
                        let _ = dev_otp;
                    }
                    auth.notice.set(Some(message.clone()));
                    feedback.set(Some(message));
                    if response.success {
                        save_pending_onboarding_draft(&pending_draft);
                        navigate(
                            &build_verify_otp_path(&response.email, OtpPurpose::Registration),
                            Default::default(),
                        );
                    }
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone().to_string()));
                    feedback.set(Some(error.to_string()));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <AuthArticle
            title=title
            subtitle=subtitle
        >
            <LocalNotice message=feedback />
            <RoleRegisterSwitcher current_role=role />
            <RegisterStepHeader role=role current_step=register_step />
            <form on:submit=on_submit class="auth-form">
                {move || if register_step.get() == 0 {
                    view! {
                        <>
                            <TextField label="Full name" value=name input_type="text" placeholder="Your full name" />
                            <EmailField label="Email" value=email />
                            <PhoneField label="Phone" value=phone />
                            <AddressAutocompleteField
                                label="Address"
                                value=address
                                placeholder="Search business or profile address"
                                input_id="register-primary-address"
                            />
                            <PasswordField label="Password" value=password placeholder="Create a password" />
                            <PasswordField
                                label="Confirm password"
                                value=password_confirmation
                                placeholder="Repeat the password"
                            />
                        </>
                    }.into_any()
                } else {
                    view! {
                        <>
                            <TextField label="Company name" value=company_name input_type="text" placeholder="Legal company or brokerage name" />
                            <AddressAutocompleteField
                                label="Company address"
                                value=company_address
                                placeholder="Search registered or operating address"
                                input_id="register-company-address"
                            />
                            {move || match role.get().as_str() {
                                "carrier" => view! {
                                    <>
                                        <TextField label="DOT number" value=dot_number input_type="text" placeholder="USDOT / DOT number" />
                                        <TextField label="MC number" value=mc_number input_type="text" placeholder="Motor carrier number" />
                                        <TextAreaField label="Equipment types" value=equipment_types placeholder="Dry van, reefer, flatbed" />
                                    </>
                                }.into_any(),
                                "shipper" => view! {
                                    <>
                                        <TextField label="Business entity ID" value=business_entity_id input_type="text" placeholder="Registration or entity ID" />
                                        <AddressAutocompleteField
                                            label="Facility address"
                                            value=facility_address
                                            placeholder="Search primary facility or warehouse address"
                                            input_id="register-facility-address"
                                        />
                                        <TextAreaField label="Fulfillment contact info" value=fulfillment_contact_info placeholder="Dispatcher or warehouse contact" />
                                    </>
                                }.into_any(),
                                "broker" => view! {
                                    <>
                                        <TextField label="FMCSA broker license" value=fmcsa_broker_license_no input_type="text" placeholder="Broker license number" />
                                        <TextField label="MC authority number" value=mc_authority_number input_type="text" placeholder="MC authority number" />
                                    </>
                                }.into_any(),
                                "freight_forwarder" => view! {
                                    <>
                                        <TextField label="Freight forwarder license" value=freight_forwarder_license input_type="text" placeholder="Forwarder license number" />
                                        <TextField label="Customs license" value=customs_license input_type="text" placeholder="Customs or clearance license" />
                                    </>
                                }.into_any(),
                                _ => ().into_any(),
                            }}
                        </>
                    }.into_any()
                }}
                <div class="auth-actions">
                    <nav class="auth-links">
                        <A href="/" attr:class="auth-link">"Back to portal"</A>
                        <A href="/auth/login" attr:class="auth-link">"Back to login"</A>
                        <A href="/auth/forgot-password" attr:class="auth-link">"Forgot password"</A>
                        <A href="/auth/verify-otp" attr:class="auth-link">"Already have an OTP?"</A>
                    </nav>
                    <div class="auth-inline-actions">
                        <button
                            type="button"
                            style=button_style("#475569")
                            disabled=move || register_step.get() == 0
                            on:click=move |_| register_step.set(0)
                        >
                            "Previous"
                        </button>
                        {move || if register_step.get() == 0 {
                            view! {
                                <button type="button" style=button_style("#0f766e") on:click=on_next>
                                    "Next"
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <button type="submit" style=button_style("#0f766e") disabled=move || is_submitting.get()>
                                    {move || if is_submitting.get() { "Creating account..." } else { "Create account" }}
                                </button>
                            }.into_any()
                        }}
                    </div>
                </div>
            </form>
        </AuthArticle>
    }
}
