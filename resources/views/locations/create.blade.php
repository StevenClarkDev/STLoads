@extends('admin-layout.app')
@section('content')
   <div class="col-xl-12 box-col-6 p-3">
    <div class="row">
        <div class="col-md-9">
            <div class="card">
                <div class="card-body">
                    <h5 class="mb-3">Loaction Add</h5>
                    <form class="card-body" method="POST" action="{{ route('locations.store') }}">
                        @csrf
                        <div class="row g-4">
                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <input type="text" id="location-name" class="form-control" placeholder="Enter Name" name="name" value="{{ old('name') }}" />
                                    <label for="location-name">Name</label>
                                </div>
                                @error('name') <small class="text-danger">{{ $message }}</small> @enderror
                            </div>
                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <select id="country_id" name="country_id" class="form-select">
                                        <option value="">-- Select Country --</option>
                                        @foreach($countries as $country)
                                            <option value="{{ $country->id }}" @selected(old('country_id') == $country->id)>
                                                {{ $country->name }}
                                            </option>
                                        @endforeach
                                    </select>
                                    <label for="country_id">Country</label>
                                </div>
                                @error('country_id') <small class="text-danger">{{ $message }}</small> @enderror
                            </div>

                            {{-- City (dependent) --}}
                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <select id="city_id" name="city_id" class="form-select" disabled>
                                        <option value="">-- Select City --</option>
                                    </select>
                                    <label for="city_id">City</label>
                                </div>
                                @error('city_id') <small class="text-danger">{{ $message }}</small> @enderror
                            </div>
                        </div>
                        <div class="d-flex flex-row-reverse gap-1 mt-2">
                            <button type="submit" class="btn btn-outline-primary">Submit</button>
                            <a href="{{ route('locations.index') }}" type="back" class="btn btn-outline-secondary">Back</a>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div> <!-- End of .row -->
</div>
@endsection
<script>
document.addEventListener('DOMContentLoaded', function () {
    const countryEl = document.getElementById('country_id');
    const cityEl    = document.getElementById('city_id');

    // If user comes back after validation error and country was selected, refetch cities
    const initialCountry = countryEl.value;
    const oldCityId = "{{ old('city_id') }}";

    function setCityLoading(isLoading) {
        cityEl.innerHTML = '';
        const opt = document.createElement('option');
        opt.value = '';
        opt.textContent = isLoading ? 'Loading cities…' : '-- Select City --';
        cityEl.appendChild(opt);
        cityEl.disabled = isLoading;
    }

    async function fetchCities(countryId, preselectCityId = null) {
        if (!countryId) {
            cityEl.innerHTML = '<option value="">-- Select City --</option>';
            cityEl.disabled = true;
            return;
        }

        setCityLoading(true);
        try {
            const url = "{{ route('api.cities.by-country', ':id') }}".replace(':id', countryId);
            const res = await fetch(url, { headers: { 'Accept': 'application/json' } });
            if (!res.ok) throw new Error('Network response was not ok');

            const cities = await res.json();

            cityEl.innerHTML = '<option value="">-- Select City --</option>';
            cities.forEach(c => {
                const o = document.createElement('option');
                o.value = c.id;
                o.textContent = c.name;
                if (preselectCityId && String(preselectCityId) === String(c.id)) o.selected = true;
                cityEl.appendChild(o);
            });

            cityEl.disabled = false;
        } catch (e) {
            cityEl.innerHTML = '<option value="">(Failed to load cities)</option>';
            cityEl.disabled = true;
            // Optional: use your Swal toast if SweetAlert2 is on the page
            if (window.Swal) {
                Swal.fire({ toast:true, icon:'error', title:'Failed to load cities', timer:2500, showConfirmButton:false, position:'top-end' });
            }
        }
    }

    countryEl.addEventListener('change', function () {
        fetchCities(this.value);
    });

    // Auto-fetch on load if a country was preselected (e.g., after validation error)
    if (initialCountry) {
        fetchCities(initialCountry, oldCityId);
    }
});
</script>
