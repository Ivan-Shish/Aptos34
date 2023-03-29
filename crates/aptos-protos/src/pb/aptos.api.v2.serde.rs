// @generated
impl serde::Serialize for GetAccountModulesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.account_address.is_empty() {
            len += 1;
        }
        if !self.module_names.is_empty() {
            len += 1;
        }
        if self.ledger_version.is_some() {
            len += 1;
        }
        if self.page_size.is_some() {
            len += 1;
        }
        if self.page_token.is_some() {
            len += 1;
        }
        if self.raw {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aptos.api.v2.GetAccountModulesRequest", len)?;
        if !self.account_address.is_empty() {
            struct_ser.serialize_field("accountAddress", &self.account_address)?;
        }
        if !self.module_names.is_empty() {
            struct_ser.serialize_field("moduleNames", &self.module_names)?;
        }
        if let Some(v) = self.ledger_version.as_ref() {
            struct_ser.serialize_field("ledgerVersion", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.page_size.as_ref() {
            struct_ser.serialize_field("pageSize", v)?;
        }
        if let Some(v) = self.page_token.as_ref() {
            struct_ser.serialize_field("pageToken", pbjson::private::base64::encode(&v).as_str())?;
        }
        if self.raw {
            struct_ser.serialize_field("raw", &self.raw)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetAccountModulesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account_address",
            "accountAddress",
            "module_names",
            "moduleNames",
            "ledger_version",
            "ledgerVersion",
            "page_size",
            "pageSize",
            "page_token",
            "pageToken",
            "raw",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccountAddress,
            ModuleNames,
            LedgerVersion,
            PageSize,
            PageToken,
            Raw,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "accountAddress" | "account_address" => Ok(GeneratedField::AccountAddress),
                            "moduleNames" | "module_names" => Ok(GeneratedField::ModuleNames),
                            "ledgerVersion" | "ledger_version" => Ok(GeneratedField::LedgerVersion),
                            "pageSize" | "page_size" => Ok(GeneratedField::PageSize),
                            "pageToken" | "page_token" => Ok(GeneratedField::PageToken),
                            "raw" => Ok(GeneratedField::Raw),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetAccountModulesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aptos.api.v2.GetAccountModulesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetAccountModulesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account_address__ = None;
                let mut module_names__ = None;
                let mut ledger_version__ = None;
                let mut page_size__ = None;
                let mut page_token__ = None;
                let mut raw__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AccountAddress => {
                            if account_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountAddress"));
                            }
                            account_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ModuleNames => {
                            if module_names__.is_some() {
                                return Err(serde::de::Error::duplicate_field("moduleNames"));
                            }
                            module_names__ = Some(map.next_value()?);
                        }
                        GeneratedField::LedgerVersion => {
                            if ledger_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ledgerVersion"));
                            }
                            ledger_version__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::PageSize => {
                            if page_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageSize"));
                            }
                            page_size__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::PageToken => {
                            if page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageToken"));
                            }
                            page_token__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Raw => {
                            if raw__.is_some() {
                                return Err(serde::de::Error::duplicate_field("raw"));
                            }
                            raw__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GetAccountModulesRequest {
                    account_address: account_address__.unwrap_or_default(),
                    module_names: module_names__.unwrap_or_default(),
                    ledger_version: ledger_version__,
                    page_size: page_size__,
                    page_token: page_token__,
                    raw: raw__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aptos.api.v2.GetAccountModulesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetAccountModulesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.modules.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aptos.api.v2.GetAccountModulesResponse", len)?;
        if !self.modules.is_empty() {
            struct_ser.serialize_field("modules", &self.modules)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetAccountModulesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "modules",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Modules,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "modules" => Ok(GeneratedField::Modules),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetAccountModulesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aptos.api.v2.GetAccountModulesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetAccountModulesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut modules__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Modules => {
                            if modules__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modules"));
                            }
                            modules__ = Some(
                                map.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(GetAccountModulesResponse {
                    modules: modules__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aptos.api.v2.GetAccountModulesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetResourcesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.addresses.is_empty() {
            len += 1;
        }
        if !self.resource_types.is_empty() {
            len += 1;
        }
        if self.ledger_version.is_some() {
            len += 1;
        }
        if self.page_size.is_some() {
            len += 1;
        }
        if self.page_token.is_some() {
            len += 1;
        }
        if self.raw {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aptos.api.v2.GetResourcesRequest", len)?;
        if !self.addresses.is_empty() {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        if !self.resource_types.is_empty() {
            struct_ser.serialize_field("resourceTypes", &self.resource_types)?;
        }
        if let Some(v) = self.ledger_version.as_ref() {
            struct_ser.serialize_field("ledgerVersion", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.page_size.as_ref() {
            struct_ser.serialize_field("pageSize", v)?;
        }
        if let Some(v) = self.page_token.as_ref() {
            struct_ser.serialize_field("pageToken", pbjson::private::base64::encode(&v).as_str())?;
        }
        if self.raw {
            struct_ser.serialize_field("raw", &self.raw)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetResourcesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "addresses",
            "resource_types",
            "resourceTypes",
            "ledger_version",
            "ledgerVersion",
            "page_size",
            "pageSize",
            "page_token",
            "pageToken",
            "raw",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Addresses,
            ResourceTypes,
            LedgerVersion,
            PageSize,
            PageToken,
            Raw,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "addresses" => Ok(GeneratedField::Addresses),
                            "resourceTypes" | "resource_types" => Ok(GeneratedField::ResourceTypes),
                            "ledgerVersion" | "ledger_version" => Ok(GeneratedField::LedgerVersion),
                            "pageSize" | "page_size" => Ok(GeneratedField::PageSize),
                            "pageToken" | "page_token" => Ok(GeneratedField::PageToken),
                            "raw" => Ok(GeneratedField::Raw),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetResourcesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aptos.api.v2.GetResourcesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetResourcesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut addresses__ = None;
                let mut resource_types__ = None;
                let mut ledger_version__ = None;
                let mut page_size__ = None;
                let mut page_token__ = None;
                let mut raw__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map.next_value()?);
                        }
                        GeneratedField::ResourceTypes => {
                            if resource_types__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resourceTypes"));
                            }
                            resource_types__ = Some(map.next_value()?);
                        }
                        GeneratedField::LedgerVersion => {
                            if ledger_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ledgerVersion"));
                            }
                            ledger_version__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::PageSize => {
                            if page_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageSize"));
                            }
                            page_size__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::PageToken => {
                            if page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageToken"));
                            }
                            page_token__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Raw => {
                            if raw__.is_some() {
                                return Err(serde::de::Error::duplicate_field("raw"));
                            }
                            raw__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GetResourcesRequest {
                    addresses: addresses__.unwrap_or_default(),
                    resource_types: resource_types__.unwrap_or_default(),
                    ledger_version: ledger_version__,
                    page_size: page_size__,
                    page_token: page_token__,
                    raw: raw__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aptos.api.v2.GetResourcesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetResourcesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.resources.is_empty() {
            len += 1;
        }
        if self.next_page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aptos.api.v2.GetResourcesResponse", len)?;
        if !self.resources.is_empty() {
            struct_ser.serialize_field("resources", &self.resources)?;
        }
        if let Some(v) = self.next_page_token.as_ref() {
            struct_ser.serialize_field("nextPageToken", pbjson::private::base64::encode(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetResourcesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resources",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Resources,
            NextPageToken,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "resources" => Ok(GeneratedField::Resources),
                            "nextPageToken" | "next_page_token" => Ok(GeneratedField::NextPageToken),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetResourcesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aptos.api.v2.GetResourcesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetResourcesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resources__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Resources => {
                            if resources__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resources"));
                            }
                            resources__ = Some(
                                map.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::NextPageToken => {
                            if next_page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextPageToken"));
                            }
                            next_page_token__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(GetResourcesResponse {
                    resources: resources__.unwrap_or_default(),
                    next_page_token: next_page_token__,
                })
            }
        }
        deserializer.deserialize_struct("aptos.api.v2.GetResourcesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MoveModuleWrapper {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.response.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aptos.api.v2.MoveModuleWrapper", len)?;
        if let Some(v) = self.response.as_ref() {
            match v {
                move_module_wrapper::Response::Parsed(v) => {
                    struct_ser.serialize_field("parsed", v)?;
                }
                move_module_wrapper::Response::Raw(v) => {
                    struct_ser.serialize_field("raw", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MoveModuleWrapper {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parsed",
            "raw",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parsed,
            Raw,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "parsed" => Ok(GeneratedField::Parsed),
                            "raw" => Ok(GeneratedField::Raw),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MoveModuleWrapper;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aptos.api.v2.MoveModuleWrapper")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MoveModuleWrapper, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut response__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Parsed => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parsed"));
                            }
                            response__ = map.next_value::<::std::option::Option<_>>()?.map(move_module_wrapper::Response::Parsed)
;
                        }
                        GeneratedField::Raw => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("raw"));
                            }
                            response__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| move_module_wrapper::Response::Raw(x.0));
                        }
                    }
                }
                Ok(MoveModuleWrapper {
                    response: response__,
                })
            }
        }
        deserializer.deserialize_struct("aptos.api.v2.MoveModuleWrapper", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResourceWrapper {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.response.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aptos.api.v2.ResourceWrapper", len)?;
        if let Some(v) = self.response.as_ref() {
            match v {
                resource_wrapper::Response::Parsed(v) => {
                    struct_ser.serialize_field("parsed", v)?;
                }
                resource_wrapper::Response::Raw(v) => {
                    struct_ser.serialize_field("raw", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResourceWrapper {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parsed",
            "raw",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parsed,
            Raw,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "parsed" => Ok(GeneratedField::Parsed),
                            "raw" => Ok(GeneratedField::Raw),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResourceWrapper;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aptos.api.v2.ResourceWrapper")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ResourceWrapper, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut response__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Parsed => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parsed"));
                            }
                            response__ = map.next_value::<::std::option::Option<_>>()?.map(resource_wrapper::Response::Parsed)
;
                        }
                        GeneratedField::Raw => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("raw"));
                            }
                            response__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| resource_wrapper::Response::Raw(x.0));
                        }
                    }
                }
                Ok(ResourceWrapper {
                    response: response__,
                })
            }
        }
        deserializer.deserialize_struct("aptos.api.v2.ResourceWrapper", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Resources {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.resources.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("aptos.api.v2.Resources", len)?;
        if !self.resources.is_empty() {
            struct_ser.serialize_field("resources", &self.resources)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Resources {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resources",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Resources,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "resources" => Ok(GeneratedField::Resources),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Resources;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct aptos.api.v2.Resources")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Resources, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resources__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Resources => {
                            if resources__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resources"));
                            }
                            resources__ = Some(
                                map.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(Resources {
                    resources: resources__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("aptos.api.v2.Resources", FIELDS, GeneratedVisitor)
    }
}
