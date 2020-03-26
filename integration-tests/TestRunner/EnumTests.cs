using System;
using System.Linq;
using Xunit;

namespace TestRunner
{
    public class EnumTests
    {
        [Fact]
        public void SimpleEnumRoundTrip()
        {
            foreach (var variant in Enum.GetValues(typeof(SimpleCEnum)).Cast<SimpleCEnum>())
            {
                SimpleCEnum result = IntegrationTests.RoundtripSimpleEnum(variant);
                Assert.Equal(variant, result);
            }
        }

        [Fact]
        public void DiscriminantEnumRoundTrip()
        {
            foreach (var variant in Enum.GetValues(typeof(EnumWithDiscriminants)).Cast<EnumWithDiscriminants>())
            {
                EnumWithDiscriminants result = IntegrationTests.RoundtripSimpleEnumWithDiscriminants(variant);
                Assert.Equal(variant, result);
            }
        }

        [Fact]
        public void GenerateDataEnum()
        {
            IDataEnum value = IntegrationTests.GenerateDataEnum();
            var baz = (DataEnum.Baz)value;
            Assert.Equal("Randal", baz.Name);
            Assert.Equal(11, baz.Value);
        }

        [Fact]
        public void DataEnumRoundTrip()
        {
            {
                var orig = new DataEnum.Foo();
                var result = (DataEnum.Foo)IntegrationTests.RoundtripDataEnum(orig);
                Assert.Equal(orig, result);
            }

            {
                var orig = new DataEnum.Bar() { Element0 = "What a cool enum!" };
                var result = (DataEnum.Bar)IntegrationTests.RoundtripDataEnum(orig);
                Assert.Equal(orig, result);
                Assert.Equal(orig.Element0, result.Element0);
            }

            {
                var orig = new DataEnum.Baz { Name = "Cool Guy McGee", Value = 69 };
                var result = (DataEnum.Baz)IntegrationTests.RoundtripDataEnum(orig);
                Assert.Equal(orig, result);
                Assert.Equal(orig.Name, result.Name);
                Assert.Equal(orig.Value, result.Value);
            }
        }
    }
}
