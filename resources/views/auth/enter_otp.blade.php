@extends('layouts.app')

@section('content')
<form action="{{ route('otp.verify') }}" method="POST">
    @csrf
    <div>
        <label>Enter the OTP sent to your email</label>
        <input type="text" name="otp" required maxlength="6">
        @error('otp')
            <p style="color:red">{{ $message }}</p>
        @enderror
    </div>
    <button type="submit">Verify OTP</button>
</form>
@endsection
