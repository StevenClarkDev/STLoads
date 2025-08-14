@extends('admin-layout.app')

@section('content')
    <div class="col-xl-12 box-col-6 p-3">
        <div class="row">
            <div class="col-md-9">
                <div class="card">
                    <div class="card-body">
                        <h5 class="mb-3">Location Edit</h5>

                        <form class="card-body" method="POST" action="{{ route('locations.update', $location->id) }}">
                            @csrf
                            @method('PUT')

                            <div class="row g-4">
                                {{-- Name --}}
                                <div class="col-md-6">
                                    <div class="form-floating form-floating-outline">
                                        <input type="text" id="location-name" class="form-control"
                                            placeholder="Enter Name" name="name"
                                            value="{{ old('name', $location->name) }}" />
                                        <label for="location-name">Name</label>
                                    </div>
                                    @error('name')
                                        <small class="text-danger">{{ $message }}</small>
                                    @enderror
                                </div>

                                {{-- Country --}}
                                <div class="col-md-6">
                                    <div class="form-floating form-floating-outline">
                                        @php
                                            $selectedCountry = old('country_id', $location->country_id);
                                        @endphp
                                        <select id="country_id" name="country_id" class="form-select">
                                            <option value="">-- Select Country --</option>
                                            @foreach ($countries as $country)
                                                <option value="{{ $country->id }}" @selected((string) $selectedCountry === (string) $country->id)>
                                                    {{ $country->name }}
                                                </option>
                                            @endforeach
                                        </select>
                                        <label for="country_id">Country</label>
                                    </div>
                                    @error('country_id')
                                        <small class="text-danger">{{ $message }}</small>
                                    @enderror
                                </div>

                                {{-- City (dependent) --}}
                                <div class="col-md-6">
                                    <div class="form-floating form-floating-outline">
                                        @php
                                            $selectedCity = old('city_id', $location->city_id);
                                        @endphp
                                        <select id="city_id" name="city_id" class="form-select">
                                            <option value="">-- Select City --</option>
                                            @foreach ($cities as $city)
                                                <option value="{{ $city->id }}" @selected((string) $selectedCity === (string) $city->id)>
                                                    {{ $city->name }}
                                                </option>
                                            @endforeach
                                        </select>
                                        <label for="city_id">City</label>
                                    </div>
                                    @error('city_id')
                                        <small class="text-danger">{{ $message }}</small>
                                    @enderror
                                </div>
                            </div>

                            <div class="d-flex flex-row-reverse gap-1 mt-2">
                                <button type="submit" class="btn btn-outline-primary">Submit</button>
                                <a href="{{ route('locations.index') }}" class="btn btn-outline-secondary">Back</a>
                            </div>
                        </form>

                    </div>
                </div>
            </div>
        </div> <!-- End of .row -->
    </div>
@endsection

<script>
document.addEventListener('DOMContentLoaded', () => {
  const countryEl = document.getElementById('country_id');
  const cityEl    = document.getElementById('city_id');

  async function fetchCities(countryId) {
    if (!countryId) {
      cityEl.innerHTML = '<option value="">-- Select City --</option>';
      cityEl.disabled = true;
      return;
    }
    cityEl.disabled = true;
    cityEl.innerHTML = '<option value="">Loading cities…</option>';

    // build a safe URL template (see earlier note)
    const url = "{{ url('/api/countries') }}/" + countryId + "/cities";
    const res = await fetch(url, { headers: { 'Accept': 'application/json' }});
    const data = await res.json();

    cityEl.innerHTML = '<option value="">-- Select City --</option>';
    data.forEach(c => {
      const opt = document.createElement('option');
      opt.value = c.id;
      opt.textContent = c.name;
      cityEl.appendChild(opt);
    });
    cityEl.disabled = false;
  }

  countryEl.addEventListener('change', () => fetchCities(countryEl.value));
});
</script>

