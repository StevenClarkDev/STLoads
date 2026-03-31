{{-- Form Select Component with Validation
    Usage:
    @include('components.form-select', [
        'name' => 'country',
        'label' => 'Country',
        'options' => $countries,
        'selected' => old('country'),
        'required' => true,
        'placeholder' => 'Select a country'
    ])
--}}

@props([
    'name',
    'label' => null,
    'options' => [],
    'selected' => null,
    'required' => false,
    'help' => null,
    'placeholder' => 'Select an option',
    'disabled' => false,
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
    
    <select 
        class="form-select {{ $class }} @error($name) is-invalid @enderror" 
        id="{{ $name }}" 
        name="{{ $name }}"
        {{ $required ? 'required' : '' }}
        {{ $disabled ? 'disabled' : '' }}
    >
        <option value="">{{ $placeholder }}</option>
        @foreach($options as $value => $text)
            <option value="{{ $value }}" {{ old($name, $selected) == $value ? 'selected' : '' }}>
                {{ $text }}
            </option>
        @endforeach
    </select>
    
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
