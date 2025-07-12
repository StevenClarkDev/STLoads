@extends('layouts.app')

@section('content')
<form action="{{ route('otp.verify') }}" method="POST">
    @csrf
    <label>Enter the OTP sent to your email</label>
    <input type="text" name="otp" required maxlength="6">
    <button type="submit">Verify OTP</button>
</form>
@endsection
