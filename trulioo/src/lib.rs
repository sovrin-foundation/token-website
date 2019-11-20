use indexmap::IndexMap;
use isahc::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use zeroize::Zeroize;

pub const TRIAL_BASE_URL: &str = "https://gateway.Trulioo.com/trial/configuration";
pub const BASE_URL: &str = "https://api.globaldatacompany.com/";
pub const API_KEY_HEADER: &str = "x-trulioo-api-key";
pub const CONFIGURATION_NAME: &str = "Identity%20Verification";

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Gender {
    #[serde(rename = "M")]
    Male,
    #[serde(rename = "F")]
    Female,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum NationalId {
    Id,
    Health,
    SocialService,
    TaxIdNumber,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum DocumentTypes {
    DrivingLicence,
    IdentityCard,
    Passport,
    ResidencePermit
}

/// Derived from https://www.iban.com/country-codes
/// ISO3166-1
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Alpha2Code {
    AU,
    AT,
    DK,
    NO,
    SE,
    TR,
    BR,
    BE,
    DE,
    NL,
    GB,
    US
}

#[derive(Clone, Debug, Zeroize)]
#[zeroize(drop)]
pub struct TruliooRequest {
    pub key: String,
    pub url: String,
}

impl TruliooRequest {
    pub async fn get_country_codes(&self) -> Result<Vec<String>, String> {
        let body = self
            .get(format!(
                "{}/configuration/v1/countrycodes/{}",
                self.url, CONFIGURATION_NAME
            ))
            .await?;
        let result: Vec<String> = serde_json::from_str(&body).map_err(|e| format!("{:?}", e))?;
        Ok(result)
    }

    pub async fn get_country_subdivisions<S: Display>(
        &self,
        country: S,
    ) -> Result<Vec<Subdivision>, String> {
        let body = self
            .get(format!(
                "{}/configuration/v1/countrysubdivisions/{}",
                self.url, country
            ))
            .await?;
        let result: Vec<Subdivision> =
            serde_json::from_str(&body).map_err(|e| format!("{:?}", e))?;
        Ok(result)
    }

    pub async fn get_fields<S: Display>(&self, country: S) -> Result<String, String> {
        let body = self
            .get(format!(
                "{}/configuration/v1/fields/{}/{}",
                self.url, CONFIGURATION_NAME, country
            ))
            .await?;
        Ok(body)
    }

    pub async fn get_recommended_fields<S: Display>(&self, country: S) -> Result<String, String> {
        let body = self
            .get(format!(
                "{}/configuration/v1/recommendedfields/{}/{}",
                self.url, CONFIGURATION_NAME, country
            ))
            .await?;
        Ok(body)
    }

    pub async fn get_consents<S: Display>(&self, country: S) -> Result<Vec<Consent>, String> {
        let body = self
            .get(format!(
                "{}/configuration/v1/consents/{}/{}",
                self.url, CONFIGURATION_NAME, country
            ))
            .await?;
        let result: Vec<String> = serde_json::from_str(&body).map_err(|e| format!("{:?}", e))?;
        let result = result
            .iter()
            .map(|c| Consent {
                name: c.to_string(),
                text: None,
                url: None,
            })
            .collect();
        Ok(result)
    }

    pub async fn get_detailed_consents<S: Display>(
        &self,
        country: S,
    ) -> Result<Vec<Consent>, String> {
        let body = self
            .get(format!(
                "{}/configuration/v1/detailedConsents/{}/{}",
                self.url, CONFIGURATION_NAME, country
            ))
            .await?;
        let result: Vec<Consent> = serde_json::from_str(&body).map_err(|e| format!("{:?}", e))?;
        Ok(result)
    }

    pub async fn get_test_entities<S: Display>(
        &self,
        country: S,
    ) -> Result<Vec<Option<Entity>>, String> {
        let body = self
            .get(format!(
                "{}/configuration/v1/testentities/{}/{}",
                self.url, CONFIGURATION_NAME, country
            ))
            .await?;
        let result: Vec<Option<Entity>> =
            serde_json::from_str(&body).map_err(|e| format!("{:?}", e))?;
        Ok(result)
    }

    pub async fn verify_identity<S: Display>(
        &self,
        request: &VerifyIdentityRequest,
    ) -> Result<VerifyIdentityResponse, String> {
        let post_body = serde_json::to_string(request).map_err(|e| format!("{:?}", e))?;
        let body = self
            .post(format!("{}/verifications/v1/verify", self.url), post_body)
            .await?;
        let result: VerifyIdentityResponse =
            serde_json::from_str(&body).map_err(|e| format!("{:?}", e))?;
        Ok(result)
    }

    pub async fn get_document_types<S: Display>(
        &self,
        country: S,
    ) -> Result<IndexMap<String, Vec<DocumentTypes>>, String> {
        let body = self
            .get(format!(
                "{}/configuration/v1/documentTypes/{}",
                self.url, country
            ))
            .await?;
        let result: IndexMap<String, Vec<DocumentTypes>> =
            serde_json::from_str(&body).map_err(|e| format!("{:?}", e))?;
        Ok(result)
    }

    async fn post(&self, url: String, request: String) -> Result<String, String> {
        let mut response = Request::post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header(API_KEY_HEADER, &self.key)
            .body(request)
            .map_err(|e| format!("{:?}", e))?
            .send_async()
            .await
            .map_err(|e| format!("{:?}", e))?;

        let body = response
            .text_async()
            .await
            .map_err(|e| format!("{:?}", e))?;
        Ok(body)
    }

    async fn get(&self, url: String) -> Result<String, String> {
        let mut response = Request::get(&url)
            .header("Accept", "application/json")
            .header(API_KEY_HEADER, &self.key)
            .body(Body::empty())
            .map_err(|e| format!("{:?}", e))?
            .send_async()
            .await
            .map_err(|e| format!("{:?}", e))?;

        let body = response
            .text_async()
            .await
            .map_err(|e| format!("{:?}", e))?;
        Ok(body)
    }
}

macro_rules! api_obj_impl {
    ($class:ident, $($rename:expr => $field:ident: $ty:ty),+) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct $class {
            $(
                #[serde(rename = $rename)]
                pub $field: $ty
            ),+
        }

        display_impl!($class, $($field),+);
    };
}
macro_rules! display_impl {
    ($class:ident, $( $field:ident ),+) => {
        impl Display for $class {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, stringify!($class))?;
                write!(f, "{{")?;
                $(
                    write!(f, "{}: {:?}", stringify!(Self.$field), self.$field)?;
                )+
                write!(f, "}}")
            }
        }
    };
}

api_obj_impl!(DataSource,
              "Name" => name: String,
              "Description" => description: String,
              "RequiredFields" => required_fields: Vec<Field>,
              "OptionalFields" => optional_fields: Vec<Field>,
              "AppendedFields" => appended_fields: Vec<Field>,
              "OutputFields" => output_fields: Vec<Field>,
              "SourceType" => source_type: String,
              "UpdateFrequency" => update_frequency: Option<String>,
              "Coverage" => coverage: String);

api_obj_impl!(Field,
              "FieldName" => name: String,
              "Type" => xtype: String);

api_obj_impl!(Subdivision,
              "Name" => name: String,
              "Code" => code: String,
              "ParentCode" => parent_code: String);

api_obj_impl!(Consent,
              "Name" => name: String,
              "Text" => text: Option<String>,
              "Url" => url: Option<String>);

api_obj_impl!(Entity,
              "PersonInfo" => person_info: Option<PersonInfo>,
              "Location" => location: Option<Location>,
              "Communication" => communication: Option<Communication>,
              "DriverLicense" => driver_license: Option<DriverLicense>,
              "Passport" => passport: Option<Passport>,
              "CountrySpecific" => country_specific: Option<IndexMap<String, IndexMap<String, String>>>);

api_obj_impl!(PersonInfo,
              "FirstGivenName" => first_given_name: Option<String>,
              "MiddleName" => middle_name: Option<String>,
              "FirstSurName" => first_surname: Option<String>,
              "DayOfBirth" => day_of_birth: Option<usize>,
              "MonthOfBirth" => month_of_birth: Option<usize>,
              "YearOfBirth" => year_of_birth: Option<usize>,
              "ISOLatin1Name" => iso_latin1_name: Option<String>,
              "Gender" => gender: Option<Gender>,
              "MinimumAge" => minimum_age: Option<usize>,
              "AdditionalFields" => additional_fields: Option<AdditionalFieldsPersonInfo>);

api_obj_impl!(Location,
              "BuildingNumber" => building_number: Option<String>,
              "BuildingName" => building_name: Option<String>,
              "UnitNumber" => unit_number: Option<String>,
              "StreetName" => street_name: Option<String>,
              "StreetType" => street_type: Option<String>,
              "City" => city: Option<String>,
              "Suburb" => suburb: Option<String>,
              "StateProvinceCode" => state_province_code: Option<String>,
              "PostalCode" => postal_code: Option<String>,
              "POBox" => po_box: Option<String>,
              "AdditionalFields" => additional_fields: Option<AdditionalFieldsLocation>);

api_obj_impl!(Communication,
              "Telephone" => telephone: Option<String>,
              "Telephone2" => telephone2: Option<String>,
              "MobileNumber" => mobile_number: Option<String>,
              "EmailAddress" => email_address: Option<String>);

api_obj_impl!(NationalIds,
              "Number" => number: Option<String>,
              "Type" => xtype: String,
              "DistrictOfIssue" => district_of_issue: Option<String>,
              "CityOfIssue" => city_of_issue: Option<String>,
              "ProvinceOfIssue" => province_of_issue: Option<String>,
              "CountyOfIssue" => county_of_issue: Option<String>);

api_obj_impl!(Passport,
              "Number" => number: String,
              "Mrz1" => mrz1: Option<String>,
              "Mrz2" => mrz2: Option<String>,
              "DayOfExpiry" => day_of_expiry: Option<usize>,
              "MonthOfExpiry" => month_of_expiry: Option<usize>,
              "YearOfExpiry" => year_of_expiry: Option<usize>);

api_obj_impl!(DriverLicense,
              "Number" => number: String,
              "State" => state: String,
              "DayOfExpiry" => day_of_expiry: usize,
              "MonthOfExpiry" => month_of_expiry: usize,
              "YearOfExpiry" => year_of_expiry: usize);

api_obj_impl!(VerifyIdentityRequest,
              "AcceptTruliooTermsAndConditions" => accept_trulioo_terms_and_conditions: bool,
              "ConfigurationName" => configuration_name: String,
              "CallBackUrl" => callback_url: String,
              "ConsentForDataSources" => consent_for_data_sources: Vec<String>,
              "CountryCode" => country_code: String,
              "CustomerReferenceID" => customer_reference_id: String,
              "DataFields" => datafields: DataFields,
              "Timeout" => timeout: Option<usize>,
              "CleansedAddress" => cleansed_address: Option<bool>);

api_obj_impl!(DataFields,
              "PersonInfo" => person_info: Option<PersonInfo>,
              "Location" => location: Option<Location>,
              "Communication" => communication: Option<Communication>,
              "DriverLicence" => driver_license: Option<DriverLicense>,
              "NationalIds" => national_ids: Option<Vec<NationalIds>>,
              "Passport" => passport: Option<Passport>,
              "CountrySpecific"=> country_specific: Option<IndexMap<String, IndexMap<String, String>>>);

api_obj_impl!(AdditionalFieldsPersonInfo,
              "FullName" => full_name: String);

api_obj_impl!(AdditionalFieldsLocation,
              "Address1" => address1: String);

api_obj_impl!(VerifyIdentityResponse,
              "TransactionID" => transaction_id: String,
              "UploadedDt" => uploaded_date: String,
              "CountryCode" => country_code: String,
              "ProductName" => product_name: String,
              "Record" => record: VerifyRecord,
              "Errors" => errors: Vec<String>);

api_obj_impl!(VerifyRecord,
              "TransactionRecordID" => id: String,
              "RecordStatus" => status: String,
              "DatasourceResults" => data_source_results: Vec<String>,
              "Errors" => errors: Vec<String>,
              "Rule" => rule: String,
              "Note" => note: String);

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use std::env;
    use std::fs;
    use std::path::Path;
    use toml;

    #[test]
    fn get_country_codes_works() {

        let request;
        if Path::new(".env").exists() {
            let config: Config = toml::from_str(&fs::read_to_string(".env").unwrap()).unwrap();
            request = config.api.into();
        } else {
            request = TruliooRequest {
                key: env::var("TRULIOO_API_KEY").unwrap(),
                url: env::var("TRULIOO_API_URL").unwrap()
            }
        }

        task::block_on(async {
            let codes = request.get_country_codes().await.unwrap();
            assert!(codes.len() > 0);
            println!("{:?}", codes);
            for country in codes {
//                let consents = request.get_detailed_consents(&country).await.unwrap();
//                println!("{:?}", consents);

//                let subdivisions = request.get_country_subdivisions(&country).await.unwrap();
//                println!("{:?}", subdivisions);

//                let testentities = request.get_test_entities(&country).await.unwrap();
//                println!("{:?}", testentities);
                let documenttypes = request.get_document_types(&country).await.unwrap();
                println!("{:?}", documenttypes);
            }
        });
    }

    #[derive(Deserialize)]
    struct Config {
        api: Api,
    }

    #[derive(Deserialize)]
    struct Api {
        url: String,
        key: String,
    }

    impl From<Api> for TruliooRequest {
        fn from(a: Api) -> Self {
            TruliooRequest {
                key: a.key,
                url: a.url,
            }
        }
    }
}
