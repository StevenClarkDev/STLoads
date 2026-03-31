{{-- Form Validation Helper Component
    Usage:
    @include('components.form-input', [
        'name' => 'email',
        'label' => 'Email Address',
        'type' => 'email',
        'value' => old('email'),
        'required' => true,
        'placeholder' => 'Enter your email'
    ])
--}}

@props([
    'name',
    'label' => null,
    'type' => 'text',
    'value' => null,
    'placeholder' => '',
    'required' => false,
    'help' => null,
    'autocomplete' => null,
    'disabled' => false,
    'readonly' => false,
    'class' => '',
    'wrapperClass' => 'mb-3'
])

<div class="{{ $wrapperClass }}">
    @if($label)
        <label for="{{ $name }}" class="form-label">
            {{ $label }}
            @if($required)
                <span class="text-danger">*</span>
            @endif
        </label>
    @endif
    
    <input 
        type="{{ $type }}" 
        class="form-control {{ $class }} @error($name) is-invalid @enderror" 
        id="{{ $name }}" 
        name="{{ $name }}" 
        value="{{ old($name, $value) }}"
        placeholder="{{ $placeholder }}"
        {{ $required ? 'required' : '' }}
        {{ $disabled ? 'disabled' : '' }}
        {{ $readonly ? 'readonly' : '' }}
        {{ $autocomplete ? "autocomplete={$autocomplete}" : '' }}
    >
    
    @error($name)
        <div class="invalid-feedback d-block">
            <i data-feather="alert-circle" class="validation-icon"></i>
            {{ $message }}
        </div>
    @enderror
    
    @if($help && !$errors->has($name))
        <small class="form-text text-muted">
            <i data-feather="info" class="help-icon"></i>
            {{ $help }}
        </small>
    @endif
</div>

@once
@push('styles')
<style>
    .validation-icon,
    .help-icon {
        width: 14px;
        height: 14px;
        margin-right: 4px;
        vertical-align: middle;
    }
    
    .form-control.is-invalid {
        border-color: #dc3545;
        padding-right: calc(1.5em + 0.75rem);
        background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 12 12' width='12' height='12' fill='none' stroke='%23dc3545'%3e%3ccircle cx='6' cy='6' r='4.5'/%3e%3cpath stroke-linejoin='round' d='M5.8 3.6h.4L6 6.5z'/%3e%3ccircle cx='6' cy='8.2' r='.6' fill='%23dc3545' stroke='none'/%3e%3c/svg%3e");
        background-repeat: no-repeat;
        background-position: right calc(0.375em + 0.1875rem) center;
        background-size: calc(0.75em + 0.375rem) calc(0.75em + 0.375rem);
    }
    
    .form-control:focus {
        border-color: #7366ff;
        box-shadow: 0 0 0 0.2rem rgba(115, 102, 255, 0.25);
    }
    
    .form-control.is-invalid:focus {
        border-color: #dc3545;
        box-shadow: 0 0 0 0.2rem rgba(220, 53, 69, 0.25);
    }
</style>
@endpush
@endonce
